use std::sync::Arc;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::config::Config;
use saleor_app_sdk::{apl::APL, manifest::AppManifest, SaleorApp};
// Make our own error that wraps `anyhow::Error`.
pub struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

pub fn trace_to_std(config: &Config) {
    tracing_subscriber::fmt()
        .with_max_level(config.log_level)
        .with_target(false)
        .init();
}

#[derive(Debug, Clone)]
pub struct AppState<A: APL> {
    pub saleor_app: Arc<tokio::sync::Mutex<SaleorApp<A>>>,
    pub config: Config,
    pub manifest: AppManifest,
}
