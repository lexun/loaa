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
    // Get base URL for WWW-Authenticate header
    let base_url = std::env::var("LOAA_BASE_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:3000".to_string());

    // Extract Authorization header
    let auth_header = req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());

    // If no Authorization header, return 401 with WWW-Authenticate header
    if auth_header.is_none() {
        let mut response = Response::new(Body::from("Unauthorized"));
        *response.status_mut() = StatusCode::UNAUTHORIZED;
        response.headers_mut().insert(
            axum::http::header::WWW_AUTHENTICATE,
            axum::http::HeaderValue::from_str(&format!(
                "Bearer resource_metadata=\"{}/.well-known/oauth-protected-resource\", scope=\"mcp:tools:read mcp:tools:write\"",
                base_url
            )).unwrap()
        );
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Check Bearer scheme
    let token = auth_header.unwrap()
        .strip_prefix("Bearer ");

    if token.is_none() {
        let mut response = Response::new(Body::from("Unauthorized: Bearer token required"));
        *response.status_mut() = StatusCode::UNAUTHORIZED;
        response.headers_mut().insert(
            axum::http::header::WWW_AUTHENTICATE,
            axum::http::HeaderValue::from_str(&format!(
                "Bearer resource_metadata=\"{}/.well-known/oauth-protected-resource\", scope=\"mcp:tools:read mcp:tools:write\"",
                base_url
            )).unwrap()
        );
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = token.unwrap();

    // Get JWT secret from environment
    let jwt_secret = std::env::var("LOAA_JWT_SECRET")
        .map_err(|_| {
            eprintln!("ERROR: LOAA_JWT_SECRET not set");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Create validation settings
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_issuer(&[base_url.clone()]);
    validation.set_audience(&["loaa-mcp"]);

    // Validate JWT
    let result = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &validation
    );

    if let Err(e) = result {
        eprintln!("JWT validation failed: {}", e);
        let mut response = Response::new(Body::from(format!("Unauthorized: {}", e)));
        *response.status_mut() = StatusCode::UNAUTHORIZED;
        response.headers_mut().insert(
            axum::http::header::WWW_AUTHENTICATE,
            axum::http::HeaderValue::from_str(&format!(
                "Bearer resource_metadata=\"{}/.well-known/oauth-protected-resource\", scope=\"mcp:tools:read mcp:tools:write\"",
                base_url
            )).unwrap()
        );
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Token is valid, proceed with request
    Ok(next.run(req).await)
}
