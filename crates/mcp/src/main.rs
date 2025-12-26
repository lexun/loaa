//! Loaa MCP Server - Standalone Binary
//!
//! This is the standalone entry point for the MCP server.
//! For embedding MCP into the web server, use the library functions directly.

use anyhow::Result;
use loaa_core::config::Config;
use loaa_mcp::{LoaaServer, run_stdio_server, run_http_server};

#[tokio::main]
async fn main() -> Result<()> {
    eprintln!("Initializing Loa'a MCP Server...");

    // Load configuration from environment
    let config = Config::from_env();
    config.validate()
        .map_err(|e| anyhow::anyhow!("Config validation error: {}", e))?;

    eprintln!("Database mode: {:?}", config.database.mode);

    // Determine transport mode from environment
    let transport_mode = std::env::var("LOAA_MCP_TRANSPORT")
        .unwrap_or_else(|_| "stdio".to_string())
        .to_lowercase();

    eprintln!("Transport mode: {}", transport_mode);

    // TODO: Get owner_id from OAuth token when wiring up user context
    let owner_id = "admin".to_string();
    let server = LoaaServer::new(&config.database, owner_id).await?;

    eprintln!("Loa'a MCP Server started successfully!");
    eprintln!("Available tools:");
    eprintln!("  - create_kid: Create a new kid");
    eprintln!("  - list_kids: List all kids");
    eprintln!("  - create_task: Create a new task");
    eprintln!("  - list_tasks: List all tasks");
    eprintln!("  - update_task: Update an existing task");
    eprintln!("  - delete_task: Delete a task");
    eprintln!("  - complete_task: Mark a task as complete");
    eprintln!("  - get_ledger: Get ledger for a kid");
    eprintln!("  - adjust_balance: Manually adjust a kid's balance");

    match transport_mode.as_str() {
        "http" | "sse" => {
            // HTTP/SSE transport for remote access
            let host = std::env::var("LOAA_MCP_HOST")
                .unwrap_or_else(|_| config.server.host.clone());
            let port = std::env::var("LOAA_MCP_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3001);

            eprintln!("⚠️  JWT authentication required for all requests");
            run_http_server(server, &host, port).await?;
        }
        "stdio" | _ => {
            // Default: stdio transport for local use
            run_stdio_server(server).await?;
        }
    }

    Ok(())
}
