#![allow(
    non_upper_case_globals,
    clippy::large_enum_variant,
    clippy::upper_case_acronyms,
    dead_code
)]
#![feature(let_chains)]
#![deny(clippy::unwrap_used, clippy::expect_used)]
mod app;
mod queries;
mod routes;

use saleor_app_sdk::{
    cargo_info,
    config::Config,
    manifest::{AppManifestBuilder, AppPermission},
    webhooks::{SyncWebhookEventType, WebhookManifestBuilder},
    SaleorApp,
};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    app::{get_active_payment_methods_from_env, trace_to_std, AppState},
    queries::event_transactions::{
        sub_payment_gateway_initialize_session, sub_transaction_cancelation_requested,
        sub_transaction_charge_requested, sub_transaction_initialize_session,
        sub_transaction_process_session, sub_transaction_refund_requested,
    },
    routes::create_routes,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::load()?;
    trace_to_std(&config)?;

    let saleor_app = SaleorApp::new(&config)?;

    let app_manifest = AppManifestBuilder::new(&config, cargo_info!())
        .add_webhook(
            WebhookManifestBuilder::new(&config)
                .set_query(sub_transaction_process_session)
                .add_sync_event(SyncWebhookEventType::TransactionProcessSession)
                .build(),
        )
        .add_webhook(
            WebhookManifestBuilder::new(&config)
                .set_query(sub_transaction_charge_requested)
                .add_sync_event(SyncWebhookEventType::TransactionChargeRequested)
                .build(),
        )
        .add_webhook(
            WebhookManifestBuilder::new(&config)
                .set_query(sub_transaction_refund_requested)
                .add_sync_event(SyncWebhookEventType::TransactionRefundRequested)
                .build(),
        )
        .add_webhook(
            WebhookManifestBuilder::new(&config)
                .set_query(sub_transaction_initialize_session)
                .add_sync_event(SyncWebhookEventType::TransactionInitializeSession)
                .build(),
        )
        .add_webhook(
            WebhookManifestBuilder::new(&config)
                .set_query(sub_payment_gateway_initialize_session)
                .add_sync_event(SyncWebhookEventType::PaymentGatewayInitializeSession)
                .build(),
        )
        .add_webhook(
            WebhookManifestBuilder::new(&config)
                .set_query(sub_transaction_cancelation_requested)
                .add_sync_event(SyncWebhookEventType::TransactionCancelationRequested)
                .build(),
        )
        // .add_webhook(
        //     WebhookManifestBuilder::new(&config)
        //         .set_query(sub_list_payment_gateways)
        //         .add_sync_event(SyncWebhookEventType::PaymentListGateways)
        //         .build(),
        // )
        .add_permissions(vec![
            AppPermission::HandlePayments,
            AppPermission::ManageOrders,
            AppPermission::ManageCheckouts,
            AppPermission::HandleCheckouts,
        ])
        .build();

    let app_state = AppState {
        active_payment_methods: get_active_payment_methods_from_env()?,
        manifest: app_manifest,
        config: config.clone(),
        saleor_app: Arc::new(Mutex::new(saleor_app)),
    };
    let app = create_routes(app_state);

    let listener = tokio::net::TcpListener::bind(
        "0.0.0.0:".to_owned()
            + config
                .app_api_base_url
                .split(':')
                .collect::<Vec<_>>()
                .get(2)
                .unwrap_or(&"3000"),
    )
    .await?;
    tracing::debug!("listening on {}", listener.local_addr()?);
    match axum::serve(listener, app).await {
        Ok(o) => Ok(o),
        Err(e) => anyhow::bail!(e),
    }
}
