use axum::{
    handler::HandlerWithoutStateExt,
    http::StatusCode,
    routing::{any, get, post},
    Router,
};

#[cfg(not(debug_assertions))]
use axum::middleware;
#[cfg(not(debug_assertions))]
use saleor_app_sdk::middleware::verify_webhook_signature::webhook_signature_verifier;

use tower_http::services::ServeDir;

use crate::app::AppState;

pub mod manifest;
pub mod register;
use manifest::manifest;
use register::register;

pub fn create_routes(state: AppState) -> Router {
    async fn handle_404() -> (StatusCode, &'static str) {
        (StatusCode::NOT_FOUND, "Not found")
    }
    let service = handle_404.into_service();

    #[cfg(not(debug_assertions))]
    let serve_dir = ServeDir::new("./public").not_found_service(service);

    // When working in workspace, cargo works relative to workspace dir, not app dir. This is
    // dev-only workaround
    #[cfg(debug_assertions)]
    let serve_dir = ServeDir::new("./bulk-price-manipulator/public").not_found_service(service);
    //TODO: Query for everything using the app auth token
    //TODO: "Failed fetching initial products: More than one channel exists, please spocify which one"
    let r = Router::new();

    #[cfg(not(debug_assertions))]
    let r = r.layer(middleware::from_fn(webhook_signature_verifier));

    r
        //handles just path, eg. localhost:3000/
        .route(
            "/",
            get(|| async { "Your app got installed successfully!" }),
        )
        //handles files, eg. localhost:3000/logo.png
        .fallback_service(serve_dir)
        .route("/api/manifest", get(manifest))
        .route("/api/register", post(register))
        .with_state(state)
}
