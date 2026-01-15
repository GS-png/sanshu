use anyhow::Result;
use rmcp::model::{ErrorData as McpError, Tool, ToolAnnotations, CallToolResult, Content};
use reqwest::header::AUTHORIZATION;
use reqwest::Client;
use serde_json::json;
use std::borrow::Cow;
use std::sync::Arc;
use std::time::Duration;

use super::types::{DocsRequest, DocsConfig, SearchResponse, SearchResult, docs_api_base_url, docs_website_url};
use crate::log_debug;
use crate::log_important;

/// Docs tool implementation
pub struct DocsTool;

impl DocsTool {
    /// Query framework documentation
    pub async fn query_docs(request: DocsRequest) -> Result<CallToolResult, McpError> {
        log_important!(info,
            "Docs query: library={}, topic={:?}, version={:?}, page={:?}",
            request.library, request.topic, request.version, request.page
        );

        let config = Self::get_config()
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to get docs config: {}", e), None))?;

        match Self::fetch_docs(&config, &request).await {
            Ok(result) => {
                log_important!(info, "Docs query success");
                Ok(CallToolResult {
                    content: vec![Content::text(result)],
                    is_error: Some(false),
                    meta: None,
                    structured_content: None,
                })
            }
            Err(e) => {
                let error_msg = format!("Docs query failed: {}", e);
                log_important!(warn, "{}", error_msg);
                Ok(CallToolResult {
                    content: vec![Content::text(error_msg)],
                    is_error: Some(true),
                    meta: None,
                    structured_content: None,
                })
            }
        }
    }

    /// Get tool definition
    pub fn get_tool_definition() -> Tool {
        let schema = json!({
            "type": "object",
            "properties": {
                "library": {
                    "type": "string",
                    "description": "Library identifier in format: owner/repo (e.g., vercel/next.js, facebook/react)"
                },
                "topic": {
                    "type": "string",
                    "description": "Query topic (optional, e.g., routing, authentication, core)"
                },
                "version": {
                    "type": "string",
                    "description": "Version number (optional, e.g., v15.1.8)"
                },
                "page": {
                    "type": "integer",
                    "description": "Page number (optional, default 1, max 10)",
                    "minimum": 1,
                    "maximum": 10
                }
            },
            "required": ["library"]
        });

        if let serde_json::Value::Object(schema_map) = schema {
            Tool {
                name: Cow::Borrowed("docs"),
                description: Some(Cow::Borrowed("Documentation lookup for libraries and frameworks.")),
                input_schema: Arc::new(schema_map),
                annotations: Some(ToolAnnotations {
                    title: Some("Docs Lookup".to_string()),
                    read_only_hint: Some(true),       // Only reads external docs
                    destructive_hint: Some(false),    // Not destructive
                    idempotent_hint: Some(true),      // Same query = same result
                    open_world_hint: Some(true),      // Interacts with external API
                }),
                icons: None,
                meta: None,
                output_schema: None,
                title: Some("Docs Lookup".to_string()),
            }
        } else {
            panic!("Schema creation failed");
        }
    }

    /// Get config
    async fn get_config() -> Result<DocsConfig> {
        let config = crate::config::load_standalone_config()
            .map_err(|e| anyhow::anyhow!("Failed to read config: {}", e))?;

        Ok(DocsConfig {
            api_key: config.mcp_config.docs_api_key,
            base_url: docs_api_base_url(),
        })
    }

    /// Fetch docs via HTTP
    async fn fetch_docs(config: &DocsConfig, request: &DocsRequest) -> Result<String> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        let url = format!("{}/docs/code/{}", config.base_url, request.library);
        log_debug!("Docs request URL: {}", url);

        let mut req_builder = client.get(&url);

        if let Some(api_key) = &config.api_key {
            req_builder = req_builder.header(AUTHORIZATION, format!("Bearer {}", api_key));
            log_debug!("Using API Key for auth");
        } else {
            log_debug!("Free mode, no API Key");
        }

        if let Some(topic) = &request.topic {
            req_builder = req_builder.query(&[("topic", topic)]);
        }
        if let Some(version) = &request.version {
            req_builder = req_builder.query(&[("version", version)]);
        }
        if let Some(page) = request.page {
            req_builder = req_builder.query(&[("page", page.to_string())]);
        }

        let response = req_builder.send().await?;
        let status = response.status();

        log_debug!("Docs response status: {}", status);

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unable to read error".to_string());

            if status.as_u16() == 404 {
                log_important!(info, "Library '{}' not found, triggering search", request.library);
                return Self::handle_not_found_with_search(config, request).await;
            }

            return Err(anyhow::anyhow!(
                "API request failed (status: {}): {}",
                status,
                Self::format_error_message(status.as_u16(), &error_text)
            ));
        }

        let response_text = response.text().await?;

        if response_text.trim().is_empty() {
            return Ok("No documentation found. Try adjusting query parameters.".to_string());
        }

        Ok(Self::format_text_response(&response_text, request))
    }

    /// Format error message
    fn format_error_message(status_code: u16, error_text: &str) -> String {
        match status_code {
            401 => "Invalid or expired API key".to_string(),
            404 => format!("Library not found: {}", error_text),
            429 => "Rate limit reached, consider configuring an API Key".to_string(),
            500..=599 => format!("Docs server error: {}", error_text),
            _ => error_text.to_string(),
        }
    }

    /// Format text response to Markdown
    fn format_text_response(content: &str, request: &DocsRequest) -> String {
        let mut output = String::new();

        output.push_str(&format!("# {} Documentation\n\n", request.library));

        if let Some(topic) = &request.topic {
            output.push_str(&format!("**Topic**: {}\n", topic));
        }
        if let Some(version) = &request.version {
            output.push_str(&format!("**Version**: {}\n", version));
        }
        if let Some(page) = request.page {
            output.push_str(&format!("**Page**: {}\n", page));
        }
        output.push_str("\n---\n\n");

        output.push_str(content);

        output.push_str(&format!("\n\n---\nSource: Docs - {}\n", request.library));

        output
    }

    /// Handle 404 error: search for candidate libraries
    async fn handle_not_found_with_search(
        config: &DocsConfig,
        request: &DocsRequest,
    ) -> Result<String> {
        let search_query = if request.library.contains('/') {
            request.library.split('/').last().unwrap_or(&request.library)
        } else {
            &request.library
        };

        log_debug!("Search query: {}", search_query);

        match Self::search_libraries(config, search_query).await {
            Ok(results) => {
                if results.is_empty() {
                    Ok(Self::format_not_found_no_suggestions(&request.library))
                } else {
                    Ok(Self::format_not_found_with_suggestions(&request.library, &results))
                }
            }
            Err(e) => {
                log_debug!("Search failed: {}", e);
                Ok(Self::format_not_found_no_suggestions(&request.library))
            }
        }
    }

    /// Search libraries
    async fn search_libraries(config: &DocsConfig, query: &str) -> Result<Vec<SearchResult>> {
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .build()?;

        let url = format!("{}/search", config.base_url);
        log_debug!("Docs search URL: {}", url);

        let mut req_builder = client.get(&url).query(&[("query", query)]);

        if let Some(api_key) = &config.api_key {
            req_builder = req_builder.header(AUTHORIZATION, format!("Bearer {}", api_key));
        }

        let response = req_builder.send().await?;
        let status = response.status();

        if !status.is_success() {
            return Err(anyhow::anyhow!("Search request failed: {}", status));
        }

        let response_text = response.text().await?;
        let search_response: SearchResponse = serde_json::from_str(&response_text)
            .map_err(|e| anyhow::anyhow!("Failed to parse search response: {}", e))?;

        Ok(search_response.results.into_iter().take(5).collect())
    }

    /// Format 404 error message (no suggestions)
    fn format_not_found_no_suggestions(library: &str) -> String {
        let website = docs_website_url();
        format!(
            "**Library \"{}\" not found**\n\n\
            Please check the library identifier. Format: `owner/repo`, e.g.:\n\
            - `vercel/next.js`\n\
            - `facebook/react`\n\
            - `spring-projects/spring-framework`\n\n\
            Tip: Search for libraries at [Docs]({})",
            library,
            website
        )
    }

    /// Format 404 error message (with suggestions)
    fn format_not_found_with_suggestions(library: &str, results: &[SearchResult]) -> String {
        let mut output = format!(
            "**Library \"{}\" not found**\n\n\
            **Suggestions**: Related libraries found, use full identifier to query:\n\n",
            library
        );

        for (idx, result) in results.iter().enumerate() {
            let lib_id = result.id.trim_start_matches('/');

            let mut info_parts = Vec::new();
            if let Some(stars) = result.stars {
                info_parts.push(format!("Stars: {}", Self::format_stars(stars)));
            }
            if let Some(trust_score) = result.trust_score {
                info_parts.push(format!("Score: {:.1}", trust_score));
            }

            let info_str = if info_parts.is_empty() {
                String::new()
            } else {
                format!(" ({})", info_parts.join(" | "))
            };

            output.push_str(&format!(
                "{}. **{}**{}\n",
                idx + 1,
                lib_id,
                info_str
            ));

            if let Some(desc) = &result.description {
                let short_desc = if desc.len() > 100 {
                    format!("{}...", &desc[..100])
                } else {
                    desc.clone()
                };
                output.push_str(&format!("   {}\n", short_desc));
            }
            output.push('\n');
        }

        output.push_str("---\n\n");
        output.push_str("Use full library identifier, e.g.:\n");
        output.push_str("```json\n");
        if let Some(first) = results.first() {
            let lib_id = first.id.trim_start_matches('/');
            output.push_str(&format!(
                "{{ \"library\": \"{}\", \"topic\": \"core\" }}\n",
                lib_id
            ));
        }
        output.push_str("```\n");

        output
    }

    /// Format stars count
    fn format_stars(stars: u64) -> String {
        if stars >= 1000 {
            format!("{:.1}K", stars as f64 / 1000.0)
        } else {
            stars.to_string()
        }
    }
}
