use anyhow::Context;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
};
use saleor_app_sdk::{
    headers::SALEOR_API_URL_HEADER,
    webhooks::{
        utils::{get_webhook_event_type, EitherWebhookType},
        SyncWebhookEventType,
    },
};
use tracing::{debug, error, info};

use crate::app::{AppError, AppState};

pub async fn webhooks(
    headers: HeaderMap,
    State(state): State<AppState>,
    body: String,
) -> Result<StatusCode, AppError> {
    debug!("/api/webhooks");
    debug!("req: {:?}", body);
    debug!("headers: {:?}", headers);

    let url = headers
        .get(SALEOR_API_URL_HEADER)
        .context("missing saleor api url header")?
        .to_str()?
        .to_owned();
    let event_type = get_webhook_event_type(&headers)?;
    match event_type {
        EitherWebhookType::Sync(a) => match a {
            SyncWebhookEventType::PaymentGatewayInitializeSession => {
                initialize_gateway(&state, &url).await?;
            }
            SyncWebhookEventType::TransactionProcessSession
            | SyncWebhookEventType::TransactionChargeRequested
            | SyncWebhookEventType::TransactionRefundRequested
            | SyncWebhookEventType::TransactionInitializeSession => {
                update_transaction_response(&state, &url).await?;
            }
            _ => (),
        },
        _ => (),
    }

    info!("got webhooks!");
    Ok(StatusCode::OK)
}

async fn initialize_gateway(state: &AppState, saleor_api_url: &str) -> anyhow::Result<()> {
    todo!()
}

async fn update_transaction_response(state: &AppState, saleor_api_url: &str) -> anyhow::Result<()> {
    todo!()
}
