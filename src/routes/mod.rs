mod manifest;
mod register;
use axum::{
    routing::{get, post},
    Router,
};
use manifest::manifest;
use register::register;

use crate::{app::AppState, saleor::APL};

pub fn create_routes<T: APL>(state: AppState<T>) -> Router<AppState<T>> {
    Router::new()
        .route("/api/manifest", get(manifest))
        .route("/api/register", post(register))
        .with_state(state)
}
