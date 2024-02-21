use axum::{extract::State, Json};

use crate::{
    app::{AppError, AppState},
    saleor::{AppManifest, APL},
};

pub async fn manifest<A: APL>(
    State(state): State<AppState<A>>,
) -> Result<Json<AppManifest>, AppError> {
    Ok(Json(state.manifest))
}
