#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use loaa_web::app::*;
    use tower_http::services::ServeDir;
    use tower_sessions::{Expiry, MemoryStore, SessionManagerLayer};
    use tower_sessions::cookie::time::Duration;

    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    // Set up session store with memory backend
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false) // Set to true in production with HTTPS
        .with_expiry(Expiry::OnInactivity(Duration::days(1))); // 24 hours

    // Serve static files BEFORE leptos routes so they take precedence
    let app = Router::new()
        .nest_service("/style", ServeDir::new("crates/web/style"))
        .nest_service("/pkg", ServeDir::new("target/site/pkg"))
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .layer(session_layer)
        .with_state(leptos_options);

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
