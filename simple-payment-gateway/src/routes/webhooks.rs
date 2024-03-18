use anyhow::Context;
use axum::{extract::State, http::HeaderMap, Json};
use cynic::{http::SurfExt, MutationBuilder};
use rust_decimal::Decimal;
use saleor_app_sdk::{
    headers::SALEOR_API_URL_HEADER,
    webhooks::{
        sync_response::{
            CancelationRequestedResult, ChargeRequestedResult,
            PaymentGatewayInitializeSessionResponse, PaymentListGatewaysResponse,
            RefundRequestedResult, TransactionCancelationRequestedResponse,
            TransactionChargeRequestedResponse, TransactionInitializeSessionResponse,
            TransactionProcessSessionResponse, TransactionRefundRequestedResponse,
            TransactionSessionResult,
        },
        utils::{get_webhook_event_type, EitherWebhookType},
        SyncWebhookEventType,
    },
};
use serde::Serialize;
use serde_json::Value;
use std::str::FromStr;
use tracing::{debug, error, info};

use crate::{
    app::{ActiveGateway, AppError, AppState, GatewayType},
    queries::{
        event_transactions::{
            TransactionCancelationRequested2, TransactionChargeRequested2,
            TransactionFlowStrategyEnum, TransactionInitializeSession2, TransactionProcessSession2,
            TransactionRefundRequested2,
        },
        mutation_transaction_update::{
            TransactionUpdate, TransactionUpdateInput, TransactionUpdateVariables,
        },
    },
};

pub async fn webhooks(
    headers: HeaderMap,
    State(state): State<AppState>,
    body: String,
) -> Result<Json<Value>, AppError> {
    debug!("/api/webhooks");
    debug!("req: {:?}", body);
    debug!("headers: {:?}", headers);

    let saleor_api_url = headers
        .get(SALEOR_API_URL_HEADER)
        .context("missing saleor api url header")?
        .to_str()?
        .to_owned();
    let event_type = get_webhook_event_type(&headers)?;
    let res: Json<Value> = match event_type {
        EitherWebhookType::Sync(a) => match a {
            SyncWebhookEventType::PaymentListGateways => {
                let gateways = state
                    .active_gateways
                    .iter()
                    .cloned()
                    .map(|g| g.gateway)
                    .collect::<Vec<_>>();
                Json::from(serde_json::to_value(PaymentListGatewaysResponse(gateways))?)
            }

            SyncWebhookEventType::TransactionCancelationRequested => {
                let data = serde_json::from_str::<TransactionCancelationRequested2>(&body)?;
                Json::from(serde_json::to_value(
                    TransactionCancelationRequestedResponse {
                        time: None,
                        psp_reference: "".to_owned(),
                        external_url: None,
                        message: None,
                        amount: data
                            .action
                            .amount
                            .and_then(|a| Decimal::from_str(&a.0).ok()),
                        result: Some(CancelationRequestedResult::CancelSuccess),
                    },
                )?)
            }
            SyncWebhookEventType::PaymentGatewayInitializeSession => Json::from(
                serde_json::to_value(PaymentGatewayInitializeSessionResponse::<u8> { data: None })?,
            ),
            SyncWebhookEventType::TransactionProcessSession => {
                let data = serde_json::from_str::<TransactionProcessSession2>(&body)?;
                Json::from(serde_json::to_value(TransactionProcessSessionResponse::<
                    u8,
                > {
                    data: None,
                    time: None,
                    psp_reference: None,
                    external_url: None,
                    message: None,
                    amount: Decimal::from_str(&data.action.amount.0)?,
                    result: match data.action.action_type {
                        TransactionFlowStrategyEnum::Charge => {
                            TransactionSessionResult::ChargeSuccess
                        }
                        TransactionFlowStrategyEnum::Authorization => {
                            TransactionSessionResult::AuthorizationSuccess
                        }
                    },
                })?)
            }
            SyncWebhookEventType::TransactionChargeRequested => {
                let data = serde_json::from_str::<TransactionChargeRequested2>(&body)?;
                Json::from(serde_json::to_value(TransactionChargeRequestedResponse {
                    time: None,
                    psp_reference: "".to_owned(),
                    external_url: None,
                    message: None,
                    amount: data
                        .action
                        .amount
                        .and_then(|a| Decimal::from_str(&a.0).ok()),
                    result: Some(ChargeRequestedResult::ChargeSuccess),
                })?)
            }
            SyncWebhookEventType::TransactionRefundRequested => {
                let data = serde_json::from_str::<TransactionRefundRequested2>(&body)?;
                Json::from(serde_json::to_value(TransactionRefundRequestedResponse {
                    time: None,
                    psp_reference: "".to_owned(),
                    external_url: None,
                    message: None,
                    amount: data
                        .action
                        .amount
                        .and_then(|a| Decimal::from_str(&a.0).ok()),
                    result: Some(RefundRequestedResult::RefundSuccess),
                })?)
            }
            SyncWebhookEventType::TransactionInitializeSession => {
                let data = serde_json::from_str::<TransactionInitializeSession2>(&body)?;
                let app = state.saleor_app.lock().await;
                let auth_data = app.apl.get(&saleor_api_url).await?;
                let operation = TransactionUpdate::build(TransactionUpdateVariables {
                    id: &data.transaction.id,
                    transaction: Some(TransactionUpdateInput {
                        message: Some(""),
                        ..Default::default()
                    }),
                });
                let mut res = surf::post(&saleor_api_url)
                    .header("authorization-bearer", auth_data.token)
                    .run_graphql(operation)
                    .await;

                let mut webhook_result = WebhookResult::Failiure;
                if let Ok(r) = &mut res
                    && let Some(data) = &mut r.data
                    && let Some(q_res) = &mut data.transaction_update
                {
                    if !q_res.errors.is_empty() {
                        q_res
                            .errors
                            .iter()
                            .for_each(|e| error!("failed update transaction, {:?}", e));
                    } else if let Some(tr) = &mut q_res.transaction {
                        tr.message = serde_json::to_string(&PaymentMethod {
                            payment_method: GatewayType::COD,
                        })?;
                        webhook_result = WebhookResult::Success;
                    }
                }

                Json::from(serde_json::to_value(
                    TransactionInitializeSessionResponse::<u8> {
                        data: None,
                        time: None,
                        psp_reference: None,
                        external_url: None,
                        message: None,
                        amount: Decimal::from_str(&data.action.amount.0)?,
                        result: match (data.action.action_type, webhook_result) {
                            (TransactionFlowStrategyEnum::Charge, WebhookResult::Success) => {
                                TransactionSessionResult::ChargeSuccess
                            }
                            (
                                TransactionFlowStrategyEnum::Authorization,
                                WebhookResult::Success,
                            ) => TransactionSessionResult::AuthorizationSuccess,
                            (TransactionFlowStrategyEnum::Charge, WebhookResult::Failiure) => {
                                TransactionSessionResult::ChargeFailiure
                            }
                            (
                                TransactionFlowStrategyEnum::Authorization,
                                WebhookResult::Failiure,
                            ) => TransactionSessionResult::AuthorizationFailure,
                        },
                    },
                )?)
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
enum WebhookResult {
    Success,
    Failiure,
}

#[derive(Serialize, Clone, Debug)]
struct PaymentMethod {
    payment_method: GatewayType,
}
