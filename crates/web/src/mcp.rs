/// MCP server integration for all-in-one deployment mode
/// This module allows the web server to optionally run the MCP server in the same process

use axum::{Router, middleware};
use loaa_core::config::Config;
use std::net::SocketAddr;
use anyhow::Result;

/// Start the MCP server on a separate port in the same process
/// This is spawned as a background task when LOAA_INCLUDE_MCP=true
pub async fn start_mcp_server(config: Config, jwt_secret: String, base_url: String) -> Result<()> {
    use loaa_core::db::{init_database_with_config, KidRepository, LedgerRepository, TaskRepository};
    use loaa_core::workflows::TaskCompletionWorkflow;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    // Import MCP server implementation from loaa-mcp
    // For now, we'll duplicate the essential parts to avoid circular dependencies

    eprintln!("🚀 Starting embedded MCP server...");

    let mcp_host = std::env::var("LOAA_MCP_HOST")
        .unwrap_or_else(|_| config.server.host.clone());
    let mcp_port = std::env::var("LOAA_MCP_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3001);

    let addr: SocketAddr = format!("{}:{}", mcp_host, mcp_port).parse()?;

    eprintln!("📡 MCP server will listen on http://{}", addr);
    eprintln!("⚠️  JWT authentication required for all MCP requests");

    // Set environment variables for the MCP server to use
    std::env::set_var("LOAA_JWT_SECRET", jwt_secret);
    std::env::set_var("LOAA_BASE_URL", base_url);

    // Initialize database
    let database = init_database_with_config(&config.database).await?;
    let task_repo = Arc::new(RwLock::new(TaskRepository::new(database.client.clone())));
    let kid_repo = Arc::new(RwLock::new(KidRepository::new(database.client.clone())));
    let ledger_repo = Arc::new(RwLock::new(LedgerRepository::new(database.client.clone())));
    let workflow = Arc::new(RwLock::new(TaskCompletionWorkflow::new(
        TaskRepository::new(database.client.clone()),
        KidRepository::new(database.client.clone()),
        LedgerRepository::new(database.client.clone()),
    )));

    // Create MCP server (we'll need to import the actual implementation)
    // For now, create a placeholder endpoint
    let app = Router::new()
        .route("/mcp", axum::routing::get(|| async { "MCP server placeholder" }))
        .layer(middleware::from_fn(validate_jwt_middleware));

    eprintln!("✓ MCP server ready on port {}", mcp_port);

    // Start the server
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.ok();
        })
        .await?;

    Ok(())
}

/// JWT validation middleware (copied from loaa-mcp/src/auth.rs)
async fn validate_jwt_middleware(
    req: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, axum::http::StatusCode> {
    use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        sub: String,
        iss: String,
        aud: String,
        exp: i64,
        iat: i64,
        scope: String,
    }

    // Extract Authorization header
    let auth_header = req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(axum::http::StatusCode::UNAUTHORIZED)?;

    // Check Bearer scheme
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(axum::http::StatusCode::UNAUTHORIZED)?;

    // Get JWT secret from environment
    let jwt_secret = std::env::var("LOAA_JWT_SECRET")
        .map_err(|_| {
            eprintln!("ERROR: LOAA_JWT_SECRET not set");
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Get expected issuer from environment
    let expected_issuer = std::env::var("LOAA_BASE_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:3000".to_string());

    // Create validation settings
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_issuer(&[expected_issuer]);
    validation.set_audience(&["loaa-mcp"]);

    // Validate JWT
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &validation
    ).map_err(|e| {
        eprintln!("JWT validation failed: {}", e);
        axum::http::StatusCode::UNAUTHORIZED
    })?;

    // Token is valid, proceed
    Ok(next.run(req).await)
}
