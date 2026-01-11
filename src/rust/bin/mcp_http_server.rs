// MCP HTTP server entry point - runs as independent process
// This mode may bypass Windsurf's subprocess detection
//
// Usage:
// 1. Start this server: sanshu-mcp-http
// 2. Configure Windsurf mcp_config.json:
//    {
//      "mcpServers": {
//        "sanshu": {
//          "serverUrl": "http://127.0.0.1:8808/sse"
//        }
//      }
//    }

use sanshu::{mcp::ZhiServer, utils::auto_init_logger, log_important};
use axum::Router;
use rmcp::transport::{StreamableHttpServerConfig, StreamableHttpService};
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    auto_init_logger()?;
    
    let port: u16 = std::env::var("MCP_HTTP_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8808);
    
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    
    log_important!(info, "Starting MCP HTTP (Streamable) server on port {}", port);

    let session_manager = Arc::new(LocalSessionManager::default());
    let server_config = StreamableHttpServerConfig {
        sse_keep_alive: Some(Duration::from_secs(30)),
        stateful_mode: true,
        cancellation_token: CancellationToken::new(),
    };

    let mcp_service = StreamableHttpService::new(
        || Ok::<_, std::io::Error>(ZhiServer::new()),
        session_manager,
        server_config,
    );

    // Keep the original /sse path for configuration compatibility
    let app = Router::new().route_service("/sse", mcp_service);
    
    log_important!(info, "MCP HTTP server ready at http://{}", addr);
    log_important!(info, "");
    log_important!(info, "=== Windsurf Configuration ===");
    log_important!(info, r#"Add to ~/.codeium/windsurf/mcp_config.json:"#);
    log_important!(info, r#"{{"mcpServers": {{"sanshu": {{"serverUrl": "http://127.0.0.1:{}/sse"}}}}}}"#, port);
    log_important!(info, "");
    
    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
