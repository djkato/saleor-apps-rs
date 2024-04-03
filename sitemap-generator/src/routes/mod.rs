use axum::{
    handler::HandlerWithoutStateExt,
    http::StatusCode,
    middleware,
    routing::{any, get, post},
    Router,
};
use saleor_app_sdk::middleware::verify_webhook_signature::webhook_signature_verifier;
use tower_http::services::ServeDir;

use crate::app::AppState;

pub mod manifest;
pub mod register;
pub mod webhooks;
use manifest::manifest;
use register::register;
use webhooks::webhooks;

pub fn create_routes(state: AppState) -> Router {
    async fn handle_404() -> (StatusCode, &'static str) {
        (StatusCode::NOT_FOUND, "Not found")
    }
    let service = handle_404.into_service();
    //TODO : Fix this relative path issue in workspaces
    let serve_dir = ServeDir::new("./public").not_found_service(service);

    //TODO Query for everything using the app auth token
    //TODO "Failed fetching initial products: More than one channel exists, please spocify which
    //one"
    Router::new()
        .route("/api/webhooks", any(webhooks))
        .layer(middleware::from_fn(webhook_signature_verifier))
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
