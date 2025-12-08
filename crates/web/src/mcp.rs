/// MCP server integration for all-in-one deployment mode
/// This module allows the web server to optionally run the MCP server in the same process

use loaa_core::config::Config;
use anyhow::Result;

/// Start the MCP server on a separate port in the same process
/// This is spawned as a background task when LOAA_INCLUDE_MCP=true
pub async fn start_mcp_server(config: Config, jwt_secret: String, base_url: String) -> Result<()> {
    eprintln!("🚀 Starting embedded MCP server...");

    let mcp_host = std::env::var("LOAA_MCP_HOST")
        .unwrap_or_else(|_| config.server.host.clone());
    let mcp_port = std::env::var("LOAA_MCP_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3001);

    eprintln!("📡 MCP server will listen on http://{}:{}", mcp_host, mcp_port);
    eprintln!("⚠️  JWT authentication required for all MCP requests");

    // Set environment variables for the MCP server to use
    std::env::set_var("LOAA_JWT_SECRET", &jwt_secret);
    std::env::set_var("LOAA_BASE_URL", &base_url);

    // Use run_http_server from loaa-mcp library
    // This handles server initialization, routing, and graceful shutdown
    let server = loaa_mcp::LoaaServer::new(&config.database).await?;

    eprintln!("✓ MCP server initialized");
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

    // Run the HTTP server from the loaa-mcp library
    // This handles all the Axum setup internally with the correct version
    loaa_mcp::run_http_server(server, &mcp_host, mcp_port).await?;

    Ok(())
}
