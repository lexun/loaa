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

/// JWT validation middleware for MCP HTTP endpoints
pub async fn validate_jwt(
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract Authorization header
    let auth_header = req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Check Bearer scheme
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Get JWT secret from environment
    let jwt_secret = std::env::var("LOAA_JWT_SECRET")
        .map_err(|_| {
            eprintln!("ERROR: LOAA_JWT_SECRET not set");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Get expected issuer from environment (should match web server's base URL)
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
        StatusCode::UNAUTHORIZED
    })?;

    // Token is valid, proceed with request
    Ok(next.run(req).await)
}
