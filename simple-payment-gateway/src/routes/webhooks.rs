use anyhow::Context;
use axum::{extract::State, http::HeaderMap, Json};
use rust_decimal::{prelude::FromPrimitive, Decimal};
use saleor_app_sdk::{
    headers::SALEOR_API_URL_HEADER,
    webhooks::{
        sync_response::{
            CancelationRequestedResult, ChargeRequestedResult,
            PaymentGatewayInitializeSessionResponse, RefundRequestedResult,
            TransactionCancelationRequestedResponse, TransactionChargeRequestedResponse,
            TransactionInitializeSessionResponse, TransactionProcessSessionResponse,
            TransactionRefundRequestedResponse, TransactionSessionResult,
        },
        utils::{get_webhook_event_type, EitherWebhookType},
        SyncWebhookEventType,
    },
};
use serde_json::Value;
use std::str::FromStr;
use tracing::{debug, error, info};

use crate::{
    app::{
        AppError, AppState, PaymentGatewayInitializeSessionData, PaymentMethodType,
        TransactionInitializeSessionData,
    },
    queries::event_transactions::{
        DeliveryMethod, OrderOrCheckout, PaymentGatewayInitializeSession2,
        TransactionCancelationRequested2, TransactionChargeRequested2, TransactionFlowStrategyEnum,
        TransactionInitializeSession2, TransactionProcessSession2, TransactionRefundRequested2,
    },
};

pub async fn webhooks(
    headers: HeaderMap,
    State(state): State<AppState>,
    body: String,
) -> Result<Json<Value>, AppError> {
    debug!("/api/webhooks");
    debug!("req: {:?}", body);

    let saleor_api_url = headers
        .get(SALEOR_API_URL_HEADER)
        .context("missing saleor api url header")?
        .to_str()?
        .to_owned();
    let event_type = get_webhook_event_type(&headers)?;

    debug!("event: {:?}", event_type);

    let res = match create_response(event_type, body, state, &saleor_api_url).await {
        Ok(r) => r,
        Err(e) => {
            error!("Response creation failed: {:?}", e);
            return Err(AppError(anyhow::anyhow!(e)));
        }
    };

    debug!("res: {}", &res.to_string());
    info!("got webhooks!");
    Ok(res)
}

async fn create_response(
    event_type: EitherWebhookType,
    body: String,
    state: AppState,
    saleor_api_url: &str,
) -> anyhow::Result<Json<Value>> {
    Ok(match event_type {
        EitherWebhookType::Sync(a) => match a {
            // SyncWebhookEventType::PaymentListGateways => {
            //     let gateways = state
            //         .active_gateways
            //         .iter()
            //         .cloned()
            //         .map(|g| g.gateway)
            //         .collect::<Vec<_>>();
            //     Json::from(serde_json::to_value(PaymentListGatewaysResponse(gateways))?)
            // }
            SyncWebhookEventType::PaymentGatewayInitializeSession => {
                let session_data = serde_json::from_str::<PaymentGatewayInitializeSession2>(&body)?;
                let mut filtered_payment_methods = state.active_payment_methods.clone();

                //If obtainment method is via some sort of shipping, remove PaymentMethodType::Cash
                //If obtainment method is collection in person at warehouse, remove PaymentMethodType::CODv
                match session_data.source_object {
                    OrderOrCheckout::Order(o) => {
                        if o.shipping_method_name.is_some() {
                            filtered_payment_methods.retain(|p| p.typ != PaymentMethodType::Cash)
                        } else if o.collection_point_name.is_some() {
                            filtered_payment_methods.retain(|p| p.typ != PaymentMethodType::COD)
                        } else {
                            error!("Order has neither shipping_method_name or collection_point_name, how is it being payed for?");
                        }
                    }
                    OrderOrCheckout::Checkout(c) => {
                        if let Some(d) = c.delivery_method {
                            match d {
                                DeliveryMethod::Warehouse(_) => {
                                    filtered_payment_methods
                                        .retain(|p| p.typ != PaymentMethodType::COD);
                                }
                                DeliveryMethod::ShippingMethod(_) => {
                                    filtered_payment_methods
                                        .retain(|p| p.typ != PaymentMethodType::Cash);
                                }
                                DeliveryMethod::Unknown => {
                                    error!("DeliveryMethod is neither");
                                }
                            }
                        }
                    }
                    OrderOrCheckout::Unknown => {
                        error!("OrderOrCheckout is neither");
                    }
                }
                let data = serde_json::to_value(PaymentGatewayInitializeSessionData {
                    payment_methods: filtered_payment_methods,
                })?;
                Json::from(serde_json::to_value(
                    PaymentGatewayInitializeSessionResponse::<Value> { data: Some(data) },
                )?)
            }
            SyncWebhookEventType::TransactionInitializeSession => {
                let session_data = serde_json::from_str::<TransactionInitializeSession2>(&body)?;

                debug!(
                    "Transaction session initialised with '{:?}' payment method.",
                    &session_data.data
                );
                let payment_method = session_data
                    .data
                    .context("Missing Payment Method in request")?
                    .payment_method;

                let apl_token = state
                    .saleor_app
                    .lock()
                    .await
                    .apl
                    .get(saleor_api_url)
                    .await?
                    .token;

                let str_payment_method =
                    serde_json::to_string(&TransactionInitializeSessionData { payment_method })?;

                // update_transaction_message(
                //     session_data.transaction.id,
                //     str_payment_method.clone(),
                //     apl_token,
                //     saleor_api_url.to_owned(),
                // );

                Json::from(serde_json::to_value(
                    TransactionInitializeSessionResponse::<u8> {
                        data: None,
                        time: None,
                        psp_reference: Some(
                            "New transaction from ".to_owned() + &state.manifest.name,
                        ),
                        external_url: None,
                        message: Some(str_payment_method),
                        amount: Decimal::from_f32(session_data.action.amount.0)
                            .context("failed to convert f32 to dec")?,
                        result: match session_data.action.action_type {
                            TransactionFlowStrategyEnum::Charge => {
                                TransactionSessionResult::ChargeSuccess
                            }
                            TransactionFlowStrategyEnum::Authorization => {
                                TransactionSessionResult::AuthorizationSuccess
                            }
                        },
                    },
                )?)
            }
            SyncWebhookEventType::TransactionChargeRequested => {
                let data = serde_json::from_str::<TransactionChargeRequested2>(&body)?;
                Json::from(serde_json::to_value(TransactionChargeRequestedResponse {
                    time: None,
                    psp_reference: "".to_owned(),
                    external_url: None,
                    message: None,
                    amount: data.action.amount.and_then(|a| Decimal::from_f32(a.0)),
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
                    amount: data.action.amount.and_then(|a| Decimal::from_f32(a.0)),
                    result: Some(RefundRequestedResult::RefundSuccess),
                })?)
            }

            SyncWebhookEventType::TransactionCancelationRequested => {
                let data = serde_json::from_str::<TransactionCancelationRequested2>(&body)?;
                Json::from(serde_json::to_value(
                    TransactionCancelationRequestedResponse {
                        time: None,
                        psp_reference: "".to_owned(),
                        external_url: None,
                        message: None,
                        amount: data.action.amount.and_then(|a| Decimal::from_f32(a.0)),
                        result: Some(CancelationRequestedResult::CancelSuccess),
                    },
                )?)
            }
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
                    amount: Decimal::from_f32(data.action.amount.0)
                        .context("failed f32 to Decimal")?,
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
            _ => Json::from(Value::from_str("")?),
        },
        _ => Json::from(Value::from_str("")?),
    })
}

// fn set_order_payment_method(
//     order_id: cynic::Id,
//     str_payment_method: String,
//     apl_token: String,
//     saleor_api_url: String,
// ) {
//     tokio::spawn(async move {
//         let operation = SetOrderPaymentMethod::build(SetOrderPaymentMethodVariables {
//             id: &trans_id,
//             transaction: Some(TransactionUpdateInput {
//                 message: Some(&str_payment_method),
//                 ..Default::default()
//             }),
//         });
//
//         debug!("operation: {:?}", serde_json::to_string(&operation));
//
//         let mut res = surf::post(saleor_api_url)
//             .header("authorization-bearer", apl_token)
//             .run_graphql(operation)
//             .await;
//
//         match &mut res {
//             Ok(r) => {
//                 if let Some(data) = &mut r.data
//                     && let Some(q_res) = &mut data.transaction_update
//                 {
//                     if !q_res.errors.is_empty() {
//                         q_res
//                             .errors
//                             .iter()
//                             .for_each(|e| error!("failed update transaction, {:?}", e));
//                     } else if q_res.transaction.is_some() {
//                         debug!("sucessfully set transactions message to payment method");
//                     }
//                 }
//             }
//             Err(e) => error!("Failed updating transaction through gql: {:?}", e),
//         }
//     });
// }
//
// enum WebhookResult {
//     Success,
//     // NeedsMessageUpdate(&'a str),
//     Failure,
// }

// fn update_transaction_message(
//     trans_id: cynic::Id,
//     str_payment_method: String,
//     apl_token: String,
//     saleor_api_url: String,
// ) {
//     tokio::spawn(async move {
//         let operation = TransactionUpdate::build(TransactionUpdateVariables {
//             id: &trans_id,
//             transaction: Some(TransactionUpdateInput {
//                 message: Some(&str_payment_method),
//                 ..Default::default()
//             }),
//         });
//
//         debug!("operation: {:?}", serde_json::to_string(&operation));
//
//         let mut res = surf::post(saleor_api_url)
//             .header("authorization-bearer", apl_token)
//             .run_graphql(operation)
//             .await;
//
//         match &mut res {
//             Ok(r) => {
//                 if let Some(data) = &mut r.data
//                     && let Some(q_res) = &mut data.transaction_update
//                 {
//                     if !q_res.errors.is_empty() {
//                         q_res
//                             .errors
//                             .iter()
//                             .for_each(|e| error!("failed update transaction, {:?}", e));
//                     } else if q_res.transaction.is_some() {
//                         debug!("sucessfully set transactions message to payment method");
//                     }
//                 }
//             }
//             Err(e) => error!("Failed updating transaction through gql: {:?}", e),
//         }
//     });
// }
//
enum WebhookResult {
    Success,
    // NeedsMessageUpdate(&'a str),
    Failure,
}
