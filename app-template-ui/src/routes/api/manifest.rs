use axum::{extract::State, Json};
use saleor_app_sdk::manifest::AppManifest;

use crate::{app::AppState, error_template::AxumError};

pub fn manifest(State(state): State<AppState>) -> Result<Json<AppManifest>, AxumError> {
    Ok(Json(state.manifest))
}
