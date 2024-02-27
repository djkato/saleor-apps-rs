use axum::{
    handler::HandlerWithoutStateExt,
    http::StatusCode,
    routing::{get, post},
    Router,
};
use saleor_app_sdk::apl::APL;
use tower_http::services::ServeDir;

use crate::app::AppState;

pub mod manifest;
pub mod register;
use manifest::manifest;
use register::register;

pub fn create_routes<T: APL + 'static>(state: AppState<T>) -> Router {
    async fn handle_404() -> (StatusCode, &'static str) {
        (StatusCode::NOT_FOUND, "Not found")
    }
    let service = handle_404.into_service();
    let serve_dir = ServeDir::new("saleor-app-template/public").not_found_service(service);

    Router::new()
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
