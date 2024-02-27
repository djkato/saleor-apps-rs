use axum::{extract::State, Json};
use saleor_app_sdk::{apl::APL, manifest::AppManifest};

use crate::app::{AppError, AppState};

pub async fn manifest<A: APL>(
    State(state): State<AppState<A>>,
) -> Result<Json<AppManifest>, AppError> {
    Ok(Json(state.manifest))
}
