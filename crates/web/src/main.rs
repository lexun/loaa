#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use axum::routing::{get, post};
    use leptos::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use loaa_web::app::*;
    use loaa_web::oauth::{
        get_authorization_server_metadata,
        get_protected_resource_metadata,
        authorize_get,
        token_post,
        OAuthState,
        AppState,
    };
    use tower_http::services::ServeDir;
    use tower_sessions::{Expiry, MemoryStore, SessionManagerLayer};
    use tower_sessions::cookie::time::Duration;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    // Set up session store with memory backend
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false) // Set to true in production with HTTPS
        .with_expiry(Expiry::OnInactivity(Duration::days(1))); // 24 hours

    // Determine base URL from environment or default
    let base_url = std::env::var("LOAA_BASE_URL")
        .unwrap_or_else(|_| format!("http://{}", addr));

    println!("🔐 OAuth base URL: {}", base_url);

    // Create combined application state
    let app_state = AppState {
        leptos_options: leptos_options.clone(),
        oauth_state: Arc::new(RwLock::new(OAuthState::new())),
        base_url,
    };

    // Serve static files BEFORE leptos routes so they take precedence
    let app = Router::new()
        // OAuth discovery endpoints
        .route(
            "/.well-known/oauth-authorization-server",
            get(get_authorization_server_metadata),
        )
        .route(
            "/.well-known/oauth-protected-resource",
            get(get_protected_resource_metadata),
        )
        // OAuth flow endpoints
        .route("/oauth/authorize", get(authorize_get))
        .route("/oauth/token", post(token_post))
        // Static files
        .nest_service("/style", ServeDir::new("crates/web/style"))
        .nest_service("/pkg", ServeDir::new("target/site/pkg"))
        // Leptos routes
        .leptos_routes(&app_state, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .layer(session_layer)
        .with_state(app_state);

    println!("🚀 Listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
}
