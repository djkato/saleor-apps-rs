use std::str::FromStr;

use anyhow::Context;
use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Response,
    Json,
};
use saleor_app_sdk::{
    headers::SALEOR_API_URL_HEADER,
    webhooks::{
        utils::{get_webhook_event_type, EitherWebhookType},
        SyncWebhookEventType,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, error, info};

use crate::{
    app::{ActiveGateway, AppError, AppState},
    queries::event_transactions::PaymentGatewayInitializeSession,
};

pub async fn webhooks(
    headers: HeaderMap,
    State(state): State<AppState>,
    body: String,
) -> Result<Json<Value>, AppError> {
    debug!("/api/webhooks");
    debug!("req: {:?}", body);
    debug!("headers: {:?}", headers);

    let url = headers
        .get(SALEOR_API_URL_HEADER)
        .context("missing saleor api url header")?
        .to_str()?
        .to_owned();
    let event_type = get_webhook_event_type(&headers)?;
    let res: Json<Value> = match event_type {
        EitherWebhookType::Sync(a) => match a {
            SyncWebhookEventType::PaymentGatewayInitializeSession => {
                Json::from(serde_json::to_value(JsonResponse {
                    data: JsonResponseData {
                        current_gateway: ActiveGateway::COD,
                    },
                })?)
            }
            SyncWebhookEventType::TransactionProcessSession
            | SyncWebhookEventType::TransactionChargeRequested
            | SyncWebhookEventType::TransactionRefundRequested
            | SyncWebhookEventType::TransactionInitializeSession => {
                update_transaction_response(&state, &url).await?;
                todo!()
            }
            _ => Json::from(Value::from_str("")?),
        },
        _ => Json::from(Value::from_str("")?),
    };

    info!("got webhooks!");
    Ok(res)
}

#[derive(Serialize, Clone, Debug)]
pub struct JsonResponse {
    data: JsonResponseData,
}

#[derive(Serialize, Clone, Debug)]
pub struct JsonResponseData {
    current_gateway: ActiveGateway,
}

async fn update_transaction_response(state: &AppState, saleor_api_url: &str) -> anyhow::Result<()> {
    todo!()
}

async fn new_transaction_response(state: &AppState, saleor_api_url: &str) -> anyhow::Result<()> {
    debug!("Creating new transaction");

    todo!()
}
