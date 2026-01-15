pub mod types;
pub mod mcp;
pub mod commands;

pub use mcp::DocsTool;
pub use types::{DocsRequest, DocsConfig};
pub use commands::{test_docs_connection, get_docs_config, save_docs_config};
