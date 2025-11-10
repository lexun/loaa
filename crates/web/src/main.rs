use axum::{
    extract::State,
    response::Html,
    routing::get,
    Router,
};
use leptos::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use loaa_web::app::App;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing_subscriber;

// Import server functions to register them
use loaa_web::server_functions;

#[derive(Clone)]
struct AppState {
    leptos_options: LeptosOptions,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Set up Leptos
    let conf = get_configuration(None)
        .expect("Failed to load Leptos configuration");
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr.clone();

    let state = AppState {
        leptos_options: leptos_options.clone(),
    };

    // Generate routes for server functions
    let routes = generate_route_list(App);

    // Build Axum router
    let app = Router::new()
        .leptos_routes_with_handler(routes, get(handler))
        .fallback(leptos_axum::file_and_error_handler(leptos_options.clone()))
        .with_state(state)
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

    // Start server
    let listener = tokio::net::TcpListener::bind(&addr).await
        .unwrap_or_else(|e| panic!("Failed to bind to address {}: {}", addr, e));
    tracing::info!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .expect("Server error");
}

async fn handler(State(state): State<AppState>) -> Html<String> {
    let handler = leptos_axum::render_app_to_stream(
        state.leptos_options.clone(),
        move || view! { <App/> },
    );
    Html(handler().await.to_string())
}
