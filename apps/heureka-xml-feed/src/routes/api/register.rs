use anyhow::Context;
use axum::{
    extract::Json,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use saleor_app_sdk::{AuthData, AuthToken};
use tracing::{debug, error, info};

use crate::{
    app::AppState,
    error_template::AxumError,
    server::event_handler::{Event, RegenerateEvent},
};

pub async fn register(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(auth_token): Json<AuthToken>,
) -> Result<StatusCode, AxumError> {
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
        domain: Some(state.clone().config.app_api_base_url),
        app_id: state.clone().manifest.id,
        saleor_api_url: saleor_api_url.clone(),
    };
    if let Err(e) = app.apl.set(auth_data).await {
        error!("{:?}", e);
        return Err(e.into());
    };

    info!("starting regeneration of db");

    state
        .task_queue_sender
        .send(Event::Regenerate(RegenerateEvent {
            saleor_api_url: saleor_api_url.clone(),
            state: state.clone(),
        }))
        .await?;

    info!("registered app for {:?}", &saleor_api_url);
    Ok(StatusCode::OK)
}
