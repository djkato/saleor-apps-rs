
use anyhow::Context;
use axum::{
    extract::Json,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use cynic::QueryBuilder;
use saleor_app_sdk::{headers::SALEOR_API_URL_HEADER, AuthData, AuthToken};
use tracing::{debug, info};

use crate::{
    app::{AppError, AppState},
    sitemap::event_handler::{Event, RegenerateEvent},
};

pub async fn register(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(auth_token): Json<AuthToken>,
) -> Result<StatusCode, AppError> {
    debug!(
        "/api/register:\nsaleor_api_url: {:?}\nauth_token: {:?}",
        &headers.get(SALEOR_API_URL_HEADER),
        &auth_token
    );

    if auth_token.auth_token.is_empty() {
        return Err(anyhow::anyhow!("missing auth_token").into());
    }
    if let Some(url) = headers.get(SALEOR_API_URL_HEADER) {
        if url.to_str()? != state.sitemap_config.allowed_host {
            debug!("register didn't come from allowed host");
            return Err(anyhow::anyhow!("Url not in allowed hosts").into());
        }
    } else {
        debug!("no url in header");
        return Err(anyhow::anyhow!("Url in header").into());
    }

    let app = state.saleor_app.lock().await;
    let saleor_api_url = headers.get("saleor-api-url").context("missing api field")?;
    let saleor_api_url = saleor_api_url.to_str()?.to_owned();
    let auth_data = AuthData {
        jwks: None,
        token: auth_token.auth_token,
        domain: Some(state.config.app_api_base_url.clone()),
        app_id: state.manifest.id.clone(),
        saleor_api_url: saleor_api_url.clone(),
    };
    app.apl.set(auth_data).await?;

    info!("registered app for{:?}", &saleor_api_url);

    //When app registers, start collecting everything of substance
    info!("Starting caching and generation process");
    let cloned_state = state.clone();

    state
        .task_queue_sender
        .send(Event::Regenerate(RegenerateEvent {
            state: cloned_state,
            saleor_api_url,
        }))
        .await?;
    Ok(StatusCode::OK)
}
