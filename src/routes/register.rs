use anyhow::Context;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Json,
};
use tracing::{debug, info};

use crate::{
    app::AppError,
    saleor::{AuthData, AuthToken, APL},
    AppState,
};

pub async fn register<A: APL>(
    headers: HeaderMap,
    Json(auth_token): Json<AuthToken>,
    State(state): State<AppState<A>>,
) -> Result<StatusCode, AppError> {
    debug!(
        "/api/register:\nsaleor_api_url:{:?}\nauth_token:{:?}",
        headers.get("saleor-api-url"),
        auth_token
    );

    if auth_token.auth_token.is_empty() {
        return Err(anyhow::anyhow!("missing auth_token").into());
    }
    let app = state.saleor_app.lock().await;
    let saleor_api_url = headers.get("saleor-api-url").context("missing api field")?;
    let saleor_api_url = saleor_api_url.to_str()?.to_owned();
    let auth_data = AuthData {
        jwks: None,
        token: auth_token.auth_token,
        domain: Some(state.config.app_api_base_url),
        app_id: state.config.saleor_app_id,
        saleor_api_url: saleor_api_url.clone(),
    };
    app.apl.set(auth_data).await?;

    info!("registered app for{:?}", &saleor_api_url);
    Ok(StatusCode::OK)
}
