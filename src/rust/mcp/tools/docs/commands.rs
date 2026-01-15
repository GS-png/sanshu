use tauri::State;
use crate::config::AppState;
use super::types::{DocsRequest, DocsConfig, TestConnectionResponse, docs_api_base_url};

/// 测试 Docs 连接
#[tauri::command]
pub async fn test_docs_connection(
    library: Option<String>,
    topic: Option<String>,
    state: State<'_, AppState>
) -> Result<TestConnectionResponse, String> {
    // 读取配置并立即释放锁
    let docs_config = {
        let config = state.config
            .lock()
            .map_err(|e| format!("获取配置失败: {}", e))?;

        DocsConfig {
            api_key: config.mcp_config.docs_api_key.clone(),
            base_url: docs_api_base_url(),
        }
    }; // config 在这里自动 drop

    // 使用用户指定的库，或默认使用 Spring Framework
    let test_library = library.unwrap_or_else(|| "spring-projects/spring-framework".to_string());
    let test_topic = topic.or_else(|| Some("core".to_string()));

    // 执行测试查询
    let test_request = DocsRequest {
        library: test_library.clone(),
        topic: test_topic,
        version: None,
        page: Some(1),
    };

    // 调用内部方法执行查询
    match execute_test_query(&docs_config, &test_request).await {
        Ok(preview) => {
            Ok(TestConnectionResponse {
                success: true,
                message: format!("连接成功! 已获取 {} 文档", test_library),
                preview: Some(preview),
            })
        }
        Err(e) => {
            Ok(TestConnectionResponse {
                success: false,
                message: format!("连接失败: {}", e),
                preview: None,
            })
        }
    }
}

/// 执行测试查询
async fn execute_test_query(
    config: &DocsConfig,
    request: &DocsRequest
) -> Result<String, String> {
    use reqwest::header::AUTHORIZATION;
    use reqwest::Client;
    use std::time::Duration;

    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    // 构建 URL
    let url = format!("{}/docs/code/{}", config.base_url, request.library);

    // 构建请求
    let mut req_builder = client.get(&url);

    // 添加 API Key (如果有)
    if let Some(api_key) = &config.api_key {
        req_builder = req_builder.header(AUTHORIZATION, format!("Bearer {}", api_key));
    }

    // 添加查询参数
    if let Some(topic) = &request.topic {
        req_builder = req_builder.query(&[("topic", topic)]);
    }
    if let Some(page) = request.page {
        req_builder = req_builder.query(&[("page", page.to_string())]);
    }

    // 发送请求
    let response = req_builder.send().await
        .map_err(|e| format!("请求失败: {}", e))?;

    let status = response.status();

    // 处理错误状态码
    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_else(|_| "无法读取错误信息".to_string());
        return Err(format_test_error(status.as_u16(), &error_text, &request.library));
    }

    // 读取响应文本 (Docs API 返回纯文本 Markdown，不是 JSON)
    let response_text = response.text().await
        .map_err(|e| format!("读取响应失败: {}", e))?;

    // 如果响应为空
    if response_text.trim().is_empty() {
        return Ok("未找到文档内容".to_string());
    }

    // 生成预览文本 (只显示前 300 个字符)
    let preview = if response_text.len() > 300 {
        // 尝试在合适的位置截断（避免截断单词）
        let truncated = &response_text[..300];
        if let Some(last_newline) = truncated.rfind('\n') {
            format!("{}...", &truncated[..last_newline])
        } else {
            format!("{}...", truncated)
        }
    } else {
        response_text
    };

    Ok(preview)
}

/// 格式化测试错误消息
fn format_test_error(status_code: u16, error_text: &str, library: &str) -> String {
    match status_code {
        401 => "API 密钥无效或已过期".to_string(),
        404 => format!("库 \"{}\" 不存在，请检查库标识符是否正确", library),
        429 => "速率限制已达上限，建议配置 API Key".to_string(),
        500..=599 => format!("Docs 服务器错误: {}", error_text),
        _ => format!("请求失败 (状态码: {}): {}", status_code, error_text),
    }
}

/// 获取 Docs 配置 (用于前端显示)
#[tauri::command]
pub async fn get_docs_config(
    state: State<'_, AppState>
) -> Result<DocsConfigResponse, String> {
    let config = state.config
        .lock()
        .map_err(|e| format!("获取配置失败: {}", e))?;
    
    Ok(DocsConfigResponse {
        api_key: config.mcp_config.docs_api_key.clone(),
    })
}

/// Docs 配置响应
#[derive(serde::Serialize)]
pub struct DocsConfigResponse {
    pub api_key: Option<String>,
}

/// 保存 Docs 配置
#[tauri::command]
pub async fn save_docs_config(
    api_key: String,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    // 更新配置
    {
        let mut config = state.config
            .lock()
            .map_err(|e| format!("获取配置失败: {}", e))?;

        // 如果 API Key 为空，设置为 None
        config.mcp_config.docs_api_key = if api_key.trim().is_empty() {
            None
        } else {
            Some(api_key.trim().to_string())
        };
    }

    // 保存配置到文件
    crate::config::save_config(&state, &app).await
        .map_err(|e| format!("保存配置失败: {}", e))?;

    Ok(())
}

