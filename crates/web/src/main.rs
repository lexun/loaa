use axum::Router;
use leptos::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use loaa_web::app::App;
use tower::ServiceBuilder;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing_subscriber;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Set up Leptos
    let conf = get_configuration(None).await
        .expect("Failed to load Leptos configuration");
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;

    // Generate routes with LocalSet for hydration
    let routes = generate_route_list(App);

    // Build Axum router
    let app = Router::new()
        .leptos_routes(&leptos_options, routes, App)
        .fallback_service(ServeDir::new(&leptos_options.site_pkg_dir))
        .with_state(leptos_options)
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

    // Start server
    let listener = tokio::net::TcpListener::bind(&addr).await
        .unwrap_or_else(|e| panic!("Failed to bind to address {}: {}", addr, e));

    tracing::info!("listening on http://{}", &addr);

    axum::serve(listener, app.into_make_service())
        .await
        .expect("Server error");
}
