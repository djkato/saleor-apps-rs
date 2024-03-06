use axum::{
    handler::HandlerWithoutStateExt,
    http::StatusCode,
    routing::{any, get, post},
    Router,
};
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
    let serve_dir = ServeDir::new("./sitemap-generator/public").not_found_service(service);

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
        .route("/api/webhooks", any(webhooks))
        .with_state(state)
}
