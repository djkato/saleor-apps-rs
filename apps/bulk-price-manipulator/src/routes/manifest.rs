use axum::{extract::State, Json};
use saleor_app_sdk::manifest::AppManifest;

use crate::app::{AppError, AppState};

pub async fn manifest(State(state): State<AppState>) -> Result<Json<AppManifest>, AppError> {
    Ok(Json(state.manifest))
}
