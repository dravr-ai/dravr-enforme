// ABOUTME: Unified CLI entry point for dravr-enforme server
// ABOUTME: Supports serve (REST+MCP), MCP stdio, and future CLI commands
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

use std::error::Error;
use std::sync::Arc;

use clap::{Parser, Subcommand};
use dravr_tronc::mcp::transport::stdio;
use dravr_tronc::server::tracing_init;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tracing::info;

use dravr_enforme_mcp::state::ServerState;
use dravr_enforme_mcp::{build_tool_registry, McpServer};
use dravr_enforme_server::router::build_router;

/// Default host for the HTTP server.
const DEFAULT_HOST: &str = "127.0.0.1";

/// Default port for the HTTP server.
const DEFAULT_PORT: u16 = 3300;

#[derive(Parser)]
#[command(
    name = "dravr-enforme-server",
    version,
    about = "Health sync orchestrator server"
)]
struct Cli {
    /// Transport mode for MCP (when no subcommand)
    #[arg(long, default_value = "http")]
    transport: String,

    /// HTTP host
    #[arg(long)]
    host: Option<String>,

    /// HTTP port
    #[arg(long)]
    port: Option<u16>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Start the REST + MCP HTTP server
    Serve {
        /// HTTP host
        #[arg(long)]
        host: Option<String>,
        /// HTTP port
        #[arg(long)]
        port: Option<u16>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let cli = Cli::parse();

    tracing_init::init_with_notifications(&cli.transport);

    let state = Arc::new(RwLock::new(ServerState::new()));
    let tools = build_tool_registry();
    let mcp_server = Arc::new(McpServer::new(
        "dravr-enforme",
        env!("CARGO_PKG_VERSION"),
        tools,
        state,
    ));

    match cli.command {
        Some(Command::Serve { host, port }) => {
            let host = host.unwrap_or_else(|| DEFAULT_HOST.to_owned());
            let port = port.unwrap_or(DEFAULT_PORT);
            serve_http(mcp_server, &host, port).await?;
        }
        None => {
            let host = cli.host.unwrap_or_else(|| DEFAULT_HOST.to_owned());
            let port = cli.port.unwrap_or(DEFAULT_PORT);
            if cli.transport == "stdio" {
                stdio::run(mcp_server).await?;
            } else {
                serve_http(mcp_server, &host, port).await?;
            }
        }
    }

    Ok(())
}

async fn serve_http(
    mcp_server: Arc<McpServer<ServerState>>,
    host: &str,
    port: u16,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let app = build_router(mcp_server);
    let addr = format!("{host}:{port}");
    let listener = TcpListener::bind(&addr).await?;

    info!("dravr-enforme server listening on {addr}");
    info!("  Health: GET http://{addr}/health");
    info!("  MCP:    POST http://{addr}/mcp");

    axum::serve(listener, app).await?;
    Ok(())
}
