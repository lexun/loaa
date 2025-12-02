use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};

/// JWT Claims structure (must match the web server's Claims)
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Issuer
    pub iss: String,
    /// Audience
    pub aud: String,
    /// Expiration time (Unix timestamp)
    pub exp: i64,
    /// Issued at (Unix timestamp)
    pub iat: i64,
    /// OAuth scopes
    pub scope: String,
}

/// Helper to create a 401 response with WWW-Authenticate header
fn unauthorized_response(base_url: &str, message: &str) -> Response {
    let mut response = Response::new(Body::from(message.to_string()));
    *response.status_mut() = StatusCode::UNAUTHORIZED;
    response.headers_mut().insert(
        axum::http::header::WWW_AUTHENTICATE,
        axum::http::HeaderValue::from_str(&format!(
            "Bearer resource_metadata=\"{}/.well-known/oauth-protected-resource\", scope=\"mcp:tools:read mcp:tools:write\"",
            base_url
        )).unwrap()
    );
    response
}

/// JWT validation middleware for MCP HTTP endpoints
pub async fn validate_jwt(
    req: Request<Body>,
    next: Next,
) -> Response {
    // Get base URL for WWW-Authenticate header
    let base_url = std::env::var("LOAA_BASE_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:3000".to_string());

    // Extract Authorization header
    let auth_header = req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());

    // If no Authorization header, return 401 with WWW-Authenticate header
    if auth_header.is_none() {
        return unauthorized_response(&base_url, "Unauthorized: No Authorization header");
    }

    // Check Bearer scheme
    let token = match auth_header.unwrap().strip_prefix("Bearer ") {
        Some(t) => t,
        None => {
            return unauthorized_response(&base_url, "Unauthorized: Bearer token required");
        }
    };

    // Get JWT secret from environment
    let jwt_secret = match std::env::var("LOAA_JWT_SECRET") {
        Ok(s) => s,
        Err(_) => {
            eprintln!("ERROR: LOAA_JWT_SECRET not set");
            let mut response = Response::new(Body::from("Internal server error"));
            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            return response;
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
        Ok(_) => {
            // Token is valid, proceed with request
            next.run(req).await
        }
        Err(e) => {
            eprintln!("JWT validation failed: {}", e);
            unauthorized_response(&base_url, &format!("Unauthorized: {}", e))
        }
    }
}
