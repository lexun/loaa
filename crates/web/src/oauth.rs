use axum::{
    extract::{State, Query},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use uuid::Uuid;

/// OAuth 2.1 Authorization Server Metadata (RFC 8414)
#[derive(Serialize)]
pub struct AuthorizationServerMetadata {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub code_challenge_methods_supported: Vec<String>,
    pub grant_types_supported: Vec<String>,
    pub response_types_supported: Vec<String>,
    pub scopes_supported: Vec<String>,
}

/// OAuth Protected Resource Metadata (for MCP discovery)
#[derive(Serialize)]
pub struct ProtectedResourceMetadata {
    pub resource: String,
    pub authorization_servers: Vec<String>,
}

/// Authorization request parameters (from Claude)
#[derive(Deserialize)]
pub struct AuthorizeParams {
    pub client_id: String,
    pub redirect_uri: String,
    pub scope: String,
    pub state: String,
    pub code_challenge: String,
    pub code_challenge_method: String,
}

/// Token request parameters (from Claude)
#[derive(Deserialize)]
pub struct TokenParams {
    pub grant_type: String,
    pub code: String,
    pub client_id: String,
    pub code_verifier: String,
    pub redirect_uri: String,
}

/// Token response
#[derive(Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
}

/// Authorization code storage
#[derive(Clone)]
pub struct AuthorizationCode {
    pub code: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub scope: String,
    pub code_challenge: String,
    pub code_challenge_method: String,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// Access token storage
#[derive(Clone)]
pub struct AccessToken {
    pub token: String,
    pub client_id: String,
    pub user_id: String,
    pub scope: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// OAuth state storage
pub struct OAuthState {
    pub codes: HashMap<String, AuthorizationCode>,
    pub tokens: HashMap<String, AccessToken>,
}

impl OAuthState {
    pub fn new() -> Self {
        Self {
            codes: HashMap::new(),
            tokens: HashMap::new(),
        }
    }

    /// Generate a new authorization code
    pub fn create_authorization_code(
        &mut self,
        client_id: String,
        redirect_uri: String,
        scope: String,
        code_challenge: String,
        code_challenge_method: String,
        user_id: String,
    ) -> String {
        let code = Uuid::new_v4().to_string();
        let now = Utc::now();

        self.codes.insert(
            code.clone(),
            AuthorizationCode {
                code: code.clone(),
                client_id,
                redirect_uri,
                scope,
                code_challenge,
                code_challenge_method,
                user_id,
                created_at: now,
                expires_at: now + Duration::minutes(10),
            },
        );

        code
    }

    /// Exchange authorization code for access token
    pub fn exchange_code(
        &mut self,
        code: &str,
        client_id: &str,
        code_verifier: &str,
        redirect_uri: &str,
    ) -> Result<AccessToken, String> {
        // Get the authorization code
        let auth_code = self.codes.get(code)
            .ok_or_else(|| "Invalid authorization code".to_string())?;

        // Check expiration
        if Utc::now() > auth_code.expires_at {
            self.codes.remove(code);
            return Err("Authorization code expired".to_string());
        }

        // Verify client_id matches
        if auth_code.client_id != client_id {
            return Err("Client ID mismatch".to_string());
        }

        // Verify redirect_uri matches
        if auth_code.redirect_uri != redirect_uri {
            return Err("Redirect URI mismatch".to_string());
        }

        // Verify PKCE code_challenge
        let computed_challenge = if auth_code.code_challenge_method == "S256" {
            let mut hasher = Sha256::new();
            hasher.update(code_verifier.as_bytes());
            let result = hasher.finalize();
            URL_SAFE_NO_PAD.encode(result)
        } else {
            // Plain method (not recommended, but spec allows it)
            code_verifier.to_string()
        };

        if computed_challenge != auth_code.code_challenge {
            return Err("Invalid code verifier".to_string());
        }

        // Generate access token
        let token = Uuid::new_v4().to_string();
        let now = Utc::now();
        let access_token = AccessToken {
            token: token.clone(),
            client_id: auth_code.client_id.clone(),
            user_id: auth_code.user_id.clone(),
            scope: auth_code.scope.clone(),
            created_at: now,
            expires_at: now + Duration::hours(24),
        };

        // Store token
        self.tokens.insert(token.clone(), access_token.clone());

        // Remove used authorization code
        self.codes.remove(code);

        Ok(access_token)
    }

    /// Validate an access token
    pub fn validate_token(&self, token: &str) -> Result<&AccessToken, String> {
        let access_token = self.tokens.get(token)
            .ok_or_else(|| "Invalid token".to_string())?;

        if Utc::now() > access_token.expires_at {
            return Err("Token expired".to_string());
        }

        Ok(access_token)
    }

    /// Clean up expired codes and tokens
    pub fn cleanup_expired(&mut self) {
        let now = Utc::now();
        self.codes.retain(|_, code| code.expires_at > now);
        self.tokens.retain(|_, token| token.expires_at > now);
    }
}

pub type SharedOAuthState = Arc<RwLock<OAuthState>>;

/// Combined application state for OAuth and Leptos
#[derive(Clone)]
pub struct AppState {
    pub leptos_options: leptos::LeptosOptions,
    pub oauth_state: SharedOAuthState,
    pub base_url: String,
}

// Implement FromRef so Leptos can extract LeptosOptions from AppState
impl axum::extract::FromRef<AppState> for leptos::LeptosOptions {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.leptos_options.clone()
    }
}

/// Get authorization server metadata
pub async fn get_authorization_server_metadata(
    State(app_state): State<AppState>,
) -> Json<AuthorizationServerMetadata> {
    Json(AuthorizationServerMetadata {
        issuer: app_state.base_url.clone(),
        authorization_endpoint: format!("{}/oauth/authorize", app_state.base_url),
        token_endpoint: format!("{}/oauth/token", app_state.base_url),
        code_challenge_methods_supported: vec!["S256".to_string()],
        grant_types_supported: vec!["authorization_code".to_string()],
        response_types_supported: vec!["code".to_string()],
        scopes_supported: vec!["mcp:tools:read".to_string(), "mcp:tools:write".to_string()],
    })
}

/// Get protected resource metadata (for MCP server)
pub async fn get_protected_resource_metadata(
    State(app_state): State<AppState>,
) -> Json<ProtectedResourceMetadata> {
    Json(ProtectedResourceMetadata {
        resource: format!("{}/mcp", app_state.base_url),
        authorization_servers: vec![app_state.base_url],
    })
}

/// OAuth authorization endpoint (GET)
/// This handles the initial authorization request from Claude
pub async fn authorize_get(
    State(app_state): State<AppState>,
    Query(params): axum::extract::Query<AuthorizeParams>,
    session: tower_sessions::Session,
) -> axum::response::Result<axum::response::Response> {
    use axum::response::{Redirect, IntoResponse};

    // Check if user is authenticated
    let user_id: Option<String> = session.get("user_id")
        .await
        .map_err(|e| (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Session error: {}", e)
        ))?;

    // If not authenticated, redirect to login with return URL
    if user_id.is_none() {
        // Store OAuth params in session for after login
        session.insert("oauth_client_id", params.client_id)
            .await
            .map_err(|e| (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Session error: {}", e)
            ))?;
        session.insert("oauth_redirect_uri", params.redirect_uri)
            .await
            .map_err(|e| (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Session error: {}", e)
            ))?;
        session.insert("oauth_scope", params.scope)
            .await
            .map_err(|e| (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Session error: {}", e)
            ))?;
        session.insert("oauth_state", params.state)
            .await
            .map_err(|e| (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Session error: {}", e)
            ))?;
        session.insert("oauth_code_challenge", params.code_challenge)
            .await
            .map_err(|e| (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Session error: {}", e)
            ))?;
        session.insert("oauth_code_challenge_method", params.code_challenge_method)
            .await
            .map_err(|e| (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Session error: {}", e)
            ))?;

        // Redirect to login (Leptos will handle rendering the login page)
        return Ok(Redirect::to("/").into_response());
    }

    // User is authenticated - show consent page
    let user_id = user_id.unwrap();

    // For now, auto-approve (later we can add a consent UI)
    // Generate authorization code
    let mut oauth_state = app_state.oauth_state.write().await;
    let code = oauth_state.create_authorization_code(
        params.client_id.clone(),
        params.redirect_uri.clone(),
        params.scope.clone(),
        params.code_challenge.clone(),
        params.code_challenge_method.clone(),
        user_id,
    );
    drop(oauth_state);

    // Redirect back to Claude with authorization code
    let redirect_url = format!(
        "{}?code={}&state={}",
        params.redirect_uri,
        code,
        params.state
    );

    Ok(Redirect::to(&redirect_url).into_response())
}

/// OAuth token endpoint (POST)
/// This exchanges the authorization code for an access token
pub async fn token_post(
    State(app_state): State<AppState>,
    axum::Form(params): axum::Form<TokenParams>,
) -> axum::response::Result<Json<TokenResponse>> {
    // Validate grant_type
    if params.grant_type != "authorization_code" {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            "Unsupported grant_type. Only 'authorization_code' is supported.".to_string(),
        ).into());
    }

    // Exchange authorization code for access token
    let mut oauth_state = app_state.oauth_state.write().await;
    let access_token = oauth_state.exchange_code(
        &params.code,
        &params.client_id,
        &params.code_verifier,
        &params.redirect_uri,
    ).map_err(|e| (
        axum::http::StatusCode::BAD_REQUEST,
        format!("Token exchange failed: {}", e),
    ))?;
    drop(oauth_state);

    // Return token response
    let expires_in = (access_token.expires_at - access_token.created_at).num_seconds();

    Ok(Json(TokenResponse {
        access_token: access_token.token,
        token_type: "Bearer".to_string(),
        expires_in,
        refresh_token: None, // We don't support refresh tokens yet
    }))
}
