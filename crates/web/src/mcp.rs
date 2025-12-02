/// MCP server integration for all-in-one deployment mode
/// This module allows the web server to optionally run the MCP server in the same process

use axum::{Router, middleware, response::IntoResponse};
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

/// Helper to create a 401 response with WWW-Authenticate header (RFC9728)
fn unauthorized_response(base_url: &str, message: &str) -> axum::response::Response {
    use axum::http::{StatusCode, header};

    let mut response = (StatusCode::UNAUTHORIZED, message.to_string()).into_response();
    response.headers_mut().insert(
        header::WWW_AUTHENTICATE,
        axum::http::HeaderValue::from_str(&format!(
            "Bearer resource_metadata=\"{}/.well-known/oauth-protected-resource\", scope=\"mcp:tools:read mcp:tools:write\"",
            base_url
        )).unwrap()
    );
    response
}

/// JWT validation middleware with WWW-Authenticate header support
async fn validate_jwt_middleware(
    req: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
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

    // Get base URL for WWW-Authenticate header
    let base_url = std::env::var("LOAA_BASE_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:3000".to_string());

    // Extract Authorization header
    let auth_header = req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());

    if auth_header.is_none() {
        return unauthorized_response(&base_url, "Unauthorized: No Authorization header");
    }

    // Check Bearer scheme
    let token = match auth_header.unwrap().strip_prefix("Bearer ") {
        Some(t) => t,
        None => return unauthorized_response(&base_url, "Unauthorized: Bearer token required"),
    };

    // Get JWT secret from environment
    let jwt_secret = match std::env::var("LOAA_JWT_SECRET") {
        Ok(s) => s,
        Err(_) => {
            eprintln!("ERROR: LOAA_JWT_SECRET not set");
            return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()).into_response();
        }
    };

    // Create validation settings
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_issuer(&[base_url.clone()]);
    validation.set_audience(&["loaa-mcp"]);

    // Validate JWT
    match decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &validation
    ) {
        Ok(_) => next.run(req).await,
        Err(e) => {
            eprintln!("JWT validation failed: {}", e);
            unauthorized_response(&base_url, &format!("Unauthorized: {}", e))
        }
    }
}
