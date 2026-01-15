use anyhow::Result;
use rmcp::{
    ServerHandler, ServiceExt, RoleServer,
    model::{ErrorData as McpError},
    transport::stdio,
    service::RequestContext,
};
use rmcp::model::*;
use std::collections::HashMap;

use super::tools::{InteractionTool, MemoryTool, AcemcpTool, DocsTool};
use super::types::{CacheRequest, StoreRequest};
use crate::mcp::tools::docs::types::DocsRequest;
use crate::config::load_standalone_config;
use crate::{log_important, log_debug};

#[derive(Clone)]
pub struct DevkitServer {
    enabled_tools: HashMap<String, bool>,
}

impl Default for DevkitServer {
    fn default() -> Self {
        Self::new()
    }
}

impl DevkitServer {
    pub fn new() -> Self {
        // Load config, use defaults on failure
        let enabled_tools = match load_standalone_config() {
            Ok(config) => config.mcp_config.tools,
            Err(e) => {
                log_important!(warn, "Failed to load config, using defaults: {}", e);
                crate::config::default_mcp_tools()
            }
        };

        Self { enabled_tools }
    }

    /// Check if tool is enabled - reads latest config
    fn is_tool_enabled(&self, tool_name: &str) -> bool {
        // Re-read config each time to get latest state
        match load_standalone_config() {
            Ok(config) => {
                let enabled = config.mcp_config.tools.get(tool_name).copied().unwrap_or(true);
                log_debug!("Tool {} status: {}", tool_name, enabled);
                enabled
            }
            Err(e) => {
                log_important!(warn, "Config read failed, using cached: {}", e);
                // Use cached config on read failure
                self.enabled_tools.get(tool_name).copied().unwrap_or(true)
            }
        }
    }
}

impl ServerHandler for DevkitServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "build-cache".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                icons: None,
                title: None,
                website_url: None,
            },
            instructions: Some("Build cache and code indexing utilities".to_string()),
        }
    }

    async fn initialize(
        &self,
        _request: InitializeRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<ServerInfo, McpError> {
        Ok(self.get_info())
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        use std::sync::Arc;
        use std::borrow::Cow;

        let mut tools = Vec::new();

        // Cache tool - stores data for async retrieval
        let cache_schema = serde_json::json!({
            "type": "object",
            "properties": {
                "message": {
                    "type": "string",
                    "description": "Data payload to cache"
                },
                "choices": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Optional metadata tags"
                },
                "format": {
                    "type": "boolean",
                    "description": "Enable structured format, defaults to true"
                }
            },
            "required": ["message"]
        });

        if let serde_json::Value::Object(ref schema_map) = cache_schema {
            tools.push(Tool {
                name: Cow::Borrowed("cache"),
                description: Some(Cow::Borrowed("Write data to start an interactive task. Returns task_id immediately. Do NOT call repeatedly - if pending, returns existing task_id. After user completes, call cache_get with task_id.")),
                input_schema: Arc::new(schema_map.clone()),
                annotations: Some(ToolAnnotations {
                    title: Some("Cache Write".to_string()),
                    read_only_hint: Some(true),
                    destructive_hint: Some(false),
                    idempotent_hint: Some(false),    // Each call creates new task
                    open_world_hint: Some(false),
                }),
                icons: None,
                meta: None,
                output_schema: None,
                title: Some("Cache Write".to_string()),
            });
        }

        // Sync cache tool - blocks until completion
        if let serde_json::Value::Object(ref schema_map) = cache_schema {
            tools.push(Tool {
                name: Cow::Borrowed("cache_sync"),
                description: Some(Cow::Borrowed("Start interactive task and wait. May return PENDING after configured timeout. If PENDING, call cache_get with task_id.")),
                input_schema: Arc::new(schema_map.clone()),
                annotations: Some(ToolAnnotations {
                    title: Some("Cache Write (Sync)".to_string()),
                    read_only_hint: Some(true),
                    destructive_hint: Some(false),
                    idempotent_hint: Some(false),
                    open_world_hint: Some(false),
                }),
                icons: None,
                meta: None,
                output_schema: None,
                title: Some("Cache Write (Sync)".to_string()),
            });
        }

        // Cache get tool - retrieves cached data
        let cache_get_schema = serde_json::json!({
            "type": "object",
            "properties": {
                "task_id": {
                    "type": "string",
                    "description": "The task_id returned by cache/cache_sync tool"
                }
            },
            "required": ["task_id"]
        });

        if let serde_json::Value::Object(schema_map) = cache_get_schema {
            tools.push(Tool {
                name: Cow::Borrowed("cache_get"),
                description: Some(Cow::Borrowed("Get result of an interactive task. Call after cache/cache_sync with task_id. Returns PENDING if not ready. Do NOT auto-poll - only call after user confirms.")),
                input_schema: Arc::new(schema_map),
                annotations: Some(ToolAnnotations {
                    title: Some("Cache Read".to_string()),
                    read_only_hint: Some(true),
                    destructive_hint: Some(false),
                    idempotent_hint: Some(true),
                    open_world_hint: Some(false),
                }),
                icons: None,
                meta: None,
                output_schema: None,
                title: Some("Cache Read".to_string()),
            });
        }

        // Memory tool - only when enabled
        if self.is_tool_enabled("store") {
            let ji_schema = serde_json::json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "description": "Operation type: store (add entry), recall (get project info)"
                    },
                    "project_path": {
                        "type": "string",
                        "description": "Project path (required)"
                    },
                    "content": {
                        "type": "string",
                        "description": "Entry content (required for store operation)"
                    },
                    "category": {
                        "type": "string",
                        "description": "Category: rule, preference, pattern, context"
                    }
                },
                "required": ["action", "project_path"]
            });

            if let serde_json::Value::Object(ref schema_map) = ji_schema {
                tools.push(Tool {
                    name: Cow::Borrowed("store"),
                    description: Some(Cow::Borrowed("Key-value storage for build configuration and project metadata")),
                    input_schema: Arc::new(schema_map.clone()),
                    annotations: Some(ToolAnnotations {
                        title: Some("Config Store".to_string()),
                        read_only_hint: Some(false),     // Can modify (store data)
                        destructive_hint: Some(false),   // Not destructive, only additive
                        idempotent_hint: Some(true),     // Storing same data is idempotent
                        open_world_hint: Some(false),    // Closed domain, local storage
                    }),
                    icons: None,
                    meta: None,
                    output_schema: None,
                    title: Some("Config Store".to_string()),
                });
            }
        }

        // Index tool - only when enabled
        if self.is_tool_enabled("index") {
            tools.push(AcemcpTool::get_tool_definition());
        }

        // Docs tool - only when enabled
        if self.is_tool_enabled("docs") {
            tools.push(DocsTool::get_tool_definition());
        }

        log_debug!("Tools returned to client: {:?}", tools.iter().map(|t| &t.name).collect::<Vec<_>>());

        Ok(ListToolsResult {
            meta: None,
            next_cursor: None,
            tools,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        log_debug!("Tool call request: {}", request.name);

        match request.name.as_ref() {
            "cache" => {
                let arguments_value = request.arguments
                    .map(serde_json::Value::Object)
                    .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

                let cache_request: CacheRequest = serde_json::from_value(arguments_value)
                    .map_err(|e| McpError::invalid_params(format!("Parameter parse error: {}", e), None))?;

                // Use async version that returns immediately
                InteractionTool::prompt_start(cache_request).await
            }
            "cache_sync" => {
                let arguments_value = request.arguments
                    .map(serde_json::Value::Object)
                    .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

                let cache_request: CacheRequest = serde_json::from_value(arguments_value)
                    .map_err(|e| McpError::invalid_params(format!("Parameter parse error: {}", e), None))?;

                InteractionTool::prompt_sync(cache_request).await
            }
            "cache_get" => {
                let arguments_value = request.arguments
                    .map(serde_json::Value::Object)
                    .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

                let task_id = arguments_value.get("task_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .ok_or_else(|| McpError::invalid_params("task_id is required".to_string(), None))?;

                InteractionTool::cache_get(task_id).await
            }
            "store" => {
                // Check if store tool is enabled
                if !self.is_tool_enabled("store") {
                    return Err(McpError::internal_error(
                        "Memory tool is disabled".to_string(),
                        None
                    ));
                }

                let arguments_value = request.arguments
                    .map(serde_json::Value::Object)
                    .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

                let store_request: StoreRequest = serde_json::from_value(arguments_value)
                    .map_err(|e| McpError::invalid_params(format!("Parameter parse error: {}", e), None))?;

                MemoryTool::store(store_request).await
            }
            "index" => {
                if !self.is_tool_enabled("index") {
                    return Err(McpError::internal_error(
                        "Search tool is disabled".to_string(),
                        None
                    ));
                }

                let arguments_value = request.arguments
                    .map(serde_json::Value::Object)
                    .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

                let acemcp_request: crate::mcp::tools::acemcp::types::AcemcpRequest = serde_json::from_value(arguments_value)
                    .map_err(|e| McpError::invalid_params(format!("Parameter parse error: {}", e), None))?;

                AcemcpTool::search_context(acemcp_request).await
            }
            "docs" => {
                if !self.is_tool_enabled("docs") {
                    return Err(McpError::internal_error(
                        "Docs tool is disabled".to_string(),
                        None
                    ));
                }

                let arguments_value = request.arguments
                    .map(serde_json::Value::Object)
                    .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

                let docs_request: DocsRequest = serde_json::from_value(arguments_value)
                    .map_err(|e| McpError::invalid_params(format!("Parameter parse error: {}", e), None))?;

                DocsTool::query_docs(docs_request).await
            }
            _ => {
                Err(McpError::invalid_request(
                    format!("Unknown tool: {}", request.name),
                    None
                ))
            }
        }
    }
}

/// Start MCP server
pub async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    let service = DevkitServer::new()
        .serve(stdio())
        .await
        .inspect_err(|e| {
            log_important!(error, "Server start failed: {}", e);
        })?;

    service.waiting().await?;
    Ok(())
}
