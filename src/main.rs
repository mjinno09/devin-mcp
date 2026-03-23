mod devin_client;
mod server;

use anyhow::Result;
use rmcp::ServiceExt;
use tracing_subscriber::EnvFilter;

use crate::server::DevinMcpServer;

fn help_text() -> String {
    format!(
        "{name} {version}
{description}

USAGE:
    {name} [OPTIONS]

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

ENVIRONMENT VARIABLES:
    DEVIN_API_KEY       (required) Devin API key for authentication
    DEVIN_API_BASE_URL  (optional) Override the Devin API base URL
    RUST_LOG            (optional) Set log level (e.g. devin_mcp=debug)

TOOLS:
    create_session    Create a new Devin session
    get_session       Get details of an existing session
    list_sessions     List all sessions
    send_message      Send a message to a session

This server communicates via MCP (Model Context Protocol) over stdio.",
        name = env!("CARGO_PKG_NAME"),
        version = env!("CARGO_PKG_VERSION"),
        description = env!("CARGO_PKG_DESCRIPTION"),
    )
}

#[tokio::main]
async fn main() -> Result<()> {
    // --help フラグ対応
    if std::env::args().any(|a| a == "--help" || a == "-h") {
        println!("{}", help_text());
        return Ok(());
    }

    // --version フラグ対応
    if std::env::args().any(|a| a == "--version" || a == "-V") {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    // 未知の引数をチェック
    let unknown_args: Vec<String> = std::env::args()
        .skip(1)
        .filter(|a| a != "--help" && a != "-h" && a != "--version" && a != "-V")
        .collect();
    if !unknown_args.is_empty() {
        eprintln!(
            "error: unknown argument(s): {}\n\n{}",
            unknown_args.join(", "),
            help_text(),
        );
        std::process::exit(1);
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
