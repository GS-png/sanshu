use std::collections::HashMap;
use std::path::PathBuf;
use tauri::{AppHandle, State};

use crate::config::{AppState, save_config};
use crate::constants::mcp;
use crate::mcp::{
    delete_history_entries_by_time_range, delete_history_entry, export_history_entry_zip,
    export_history_by_time_range_zip, get_history_entry, history_base_dir, list_history_entries,
    HistoryEntryDetail, HistoryEntrySummary,
};
// use crate::mcp::tools::acemcp; // 已迁移到独立模块

/// MCP工具配置
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct MCPToolConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub can_disable: bool,
    pub icon: String,
    pub icon_bg: String,
    pub dark_icon_bg: String,
    pub has_config: bool, // 是否有配置选项
}

/// 获取MCP工具配置列表
#[tauri::command]
pub async fn get_mcp_tools_config(state: State<'_, AppState>) -> Result<Vec<MCPToolConfig>, String> {
    let config = state.config.lock().map_err(|e| format!("获取配置失败: {}", e))?;
    
    // 动态构建工具配置列表
    let mut tools = Vec::new();
    
    // Cache tool
    tools.push(MCPToolConfig {
        id: mcp::TOOL_CACHE.to_string(),
        name: "Cache".to_string(),
        description: "Build cache for data storage and retrieval".to_string(),
        enabled: config.mcp_config.tools.get(mcp::TOOL_CACHE).copied().unwrap_or(true),
        can_disable: false,
        icon: "i-carbon-data-backup text-lg text-blue-600 dark:text-blue-400".to_string(),
        icon_bg: "bg-blue-100 dark:bg-blue-900".to_string(),
        dark_icon_bg: "dark:bg-blue-800".to_string(),
        has_config: false,
    });
    
    // Store tool
    tools.push(MCPToolConfig {
        id: mcp::TOOL_STORE.to_string(),
        name: "Store".to_string(),
        description: "Key-value store for project configuration".to_string(),
        enabled: config.mcp_config.tools.get(mcp::TOOL_STORE).copied().unwrap_or(true),
        can_disable: true,
        icon: "i-carbon-data-base text-lg text-purple-600 dark:text-purple-400".to_string(),
        icon_bg: "bg-green-100 dark:bg-green-900".to_string(),
        dark_icon_bg: "dark:bg-green-800".to_string(),
        has_config: false,
    });
    
    // Index tool
    tools.push(MCPToolConfig {
        id: mcp::TOOL_INDEX.to_string(),
        name: "Index".to_string(),
        description: "Code indexing for fast file lookup".to_string(),
        enabled: config.mcp_config.tools.get(mcp::TOOL_INDEX).copied().unwrap_or(false),
        can_disable: true,
        icon: "i-carbon-search text-lg text-green-600 dark:text-green-400".to_string(),
        icon_bg: "bg-green-100 dark:bg-green-900".to_string(),
        dark_icon_bg: "dark:bg-green-800".to_string(),
        has_config: true, // 代码搜索工具有配置选项
    });

    // Docs tool
    tools.push(MCPToolConfig {
        id: mcp::TOOL_DOCS.to_string(),
        name: "Docs".to_string(),
        description: "Documentation lookup for libraries".to_string(),
        enabled: config.mcp_config.tools.get(mcp::TOOL_DOCS).copied().unwrap_or(true),
        can_disable: true,
        icon: "i-carbon-document text-lg text-orange-600 dark:text-orange-400".to_string(),
        icon_bg: "bg-orange-100 dark:bg-orange-900".to_string(),
        dark_icon_bg: "dark:bg-orange-800".to_string(),
        has_config: true, // Docs 工具有配置选项
    });

    // 按启用状态排序，启用的在前
    tools.sort_by(|a, b| b.enabled.cmp(&a.enabled));
    
    Ok(tools)
}

/// 设置MCP工具启用状态
#[tauri::command]
pub async fn set_mcp_tool_enabled(
    tool_id: String,
    enabled: bool,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    {
        let mut config = state.config.lock().map_err(|e| format!("获取配置失败: {}", e))?;
        
        // 检查工具是否可以禁用
        if tool_id == mcp::TOOL_CACHE && !enabled {
            return Err("Cache tool is required and cannot be disabled".to_string());
        }
        
        // 更新工具状态
        config.mcp_config.tools.insert(tool_id.clone(), enabled);
    }
    
    // 保存配置
    save_config(&state, &app).await
        .map_err(|e| format!("保存配置失败: {}", e))?;

    // 使用日志记录状态变更（在 MCP 模式下会自动输出到文件）
    log::info!("MCP工具 {} 状态已更新为: {}", tool_id, enabled);

    Ok(())
}

/// 获取所有MCP工具状态
#[tauri::command]
pub async fn get_mcp_tools_status(state: State<'_, AppState>) -> Result<HashMap<String, bool>, String> {
    let config = state.config.lock().map_err(|e| format!("获取配置失败: {}", e))?;
    Ok(config.mcp_config.tools.clone())
}

/// 重置MCP工具配置为默认值
#[tauri::command]
pub async fn reset_mcp_tools_config(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    {
        let mut config = state.config.lock().map_err(|e| format!("获取配置失败: {}", e))?;
        let default_config = mcp::get_default_mcp_config();
        config.mcp_config.tools.clear();
        for tool in &default_config.tools {
            config.mcp_config.tools.insert(tool.tool_id.clone(), tool.enabled);
        }
    }
    
    // 保存配置
    save_config(&state, &app).await
        .map_err(|e| format!("保存配置失败: {}", e))?;

    // 使用日志记录配置重置（在 MCP 模式下会自动输出到文件）
    log::info!("MCP工具配置已重置为默认值");
    Ok(())
}

/// 获取交互等待阈值（ms）
#[tauri::command]
pub async fn get_interaction_wait_ms(state: State<'_, AppState>) -> Result<u64, String> {
    let config = state.config.lock().map_err(|e| format!("获取配置失败: {}", e))?;
    Ok(config.mcp_config.interaction_wait_ms)
}

/// 设置交互等待阈值（ms）
#[tauri::command]
pub async fn set_interaction_wait_ms(
    wait_ms: u64,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    {
        let mut config = state.config.lock().map_err(|e| format!("获取配置失败: {}", e))?;
        config.mcp_config.interaction_wait_ms = wait_ms;
    }

    save_config(&state, &app)
        .await
        .map_err(|e| format!("保存配置失败: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn list_bistro_journal_entries(limit: Option<u32>) -> Result<Vec<HistoryEntrySummary>, String> {
    let limit = limit.unwrap_or(200).min(2000) as usize;
    list_history_entries(limit).map_err(|e| format!("获取历史记录失败: {}", e))
}

#[tauri::command]
pub async fn get_bistro_journal_entry(id: String) -> Result<HistoryEntryDetail, String> {
    get_history_entry(id).map_err(|e| format!("获取历史详情失败: {}", e))
}

#[tauri::command]
pub async fn delete_bistro_journal_entry(id: String) -> Result<(), String> {
    delete_history_entry(id).map_err(|e| format!("删除历史记录失败: {}", e))
}

#[tauri::command]
pub async fn delete_bistro_journal_by_time_range(
    start: Option<String>,
    end: Option<String>,
) -> Result<u32, String> {
    delete_history_entries_by_time_range(start, end)
        .map_err(|e| format!("按时间段删除失败: {}", e))
}

#[tauri::command]
pub async fn export_bistro_journal_entry_zip(id: String) -> Result<String, String> {
    let target_dir: PathBuf = dirs::download_dir()
        .or_else(dirs::data_dir)
        .or_else(dirs::config_dir)
        .unwrap_or_else(|| history_base_dir().unwrap_or_else(|_| PathBuf::from(".")));

    export_history_entry_zip(id, target_dir)
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| format!("导出失败: {}", e))
}

#[tauri::command]
pub async fn export_bistro_journal_by_time_range_zip(
    start: Option<String>,
    end: Option<String>,
) -> Result<String, String> {
    let target_dir: PathBuf = dirs::download_dir()
        .or_else(dirs::data_dir)
        .or_else(dirs::config_dir)
        .unwrap_or_else(|| history_base_dir().unwrap_or_else(|_| PathBuf::from(".")));

    export_history_by_time_range_zip(start, end, target_dir)
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| format!("导出失败: {}", e))
}

// acemcp 相关命令已迁移

// 已移除 Python Web 服务相关函数，完全使用 Rust 实现
// 如需调试配置，请直接查看本地配置文件
