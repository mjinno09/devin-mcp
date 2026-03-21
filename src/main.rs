mod devin_client;
mod server;

use anyhow::Result;
use rmcp::ServiceExt;
use tracing_subscriber::EnvFilter;

use crate::server::DevinMcpServer;

#[tokio::main]
async fn main() -> Result<()> {
    // --version フラグ対応
    if std::env::args().any(|a| a == "--version" || a == "-V") {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    // ログは必ず stderr へ（stdout は JSON-RPC 専用）
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("devin_mcp=info".parse()?))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting Devin MCP Server");

    let api_key =
        std::env::var("DEVIN_API_KEY").expect("DEVIN_API_KEY environment variable is required");

    let mut server = DevinMcpServer::new(api_key);
    if let Ok(base_url) = std::env::var("DEVIN_API_BASE_URL") {
        server.client.base_url = base_url;
    }
    let transport = rmcp::transport::io::stdio();
    let handle = server.serve(transport).await?;

    handle.waiting().await?;
    Ok(())
}
