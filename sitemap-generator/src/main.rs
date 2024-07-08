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
mod sitemap;
mod test;

use saleor_app_sdk::{
    config::Config,
    manifest::{cargo_info, AppManifestBuilder, AppPermission},
    webhooks::{AsyncWebhookEventType, WebhookManifestBuilder},
    SaleorApp,
};
use std::sync::Arc;
use tokio::{
    spawn,
    sync::{
        mpsc::{channel, Receiver},
        Mutex,
    },
};
use tracing::{debug, error, info};

use crate::{
    app::{trace_to_std, AppState, SitemapConfig},
    queries::event_subjects_updated::EVENTS_QUERY,
    routes::create_routes,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::load()?;
    trace_to_std(&config)?;
    let sitemap_config = SitemapConfig::load()?;
    debug!("Creating configs...");

    let saleor_app = SaleorApp::new(&config)?;

    debug!("Creating saleor App...");
    let app_manifest = AppManifestBuilder::new(&config, cargo_info!())
        .add_permissions(vec![
            AppPermission::ManageProducts,
            AppPermission::ManagePages,
        ])
        .add_webhook(
            WebhookManifestBuilder::new(&config)
                .set_query(EVENTS_QUERY)
                .add_async_events(vec![
                    AsyncWebhookEventType::ProductCreated,
                    AsyncWebhookEventType::ProductUpdated,
                    AsyncWebhookEventType::ProductDeleted,
                    AsyncWebhookEventType::CategoryCreated,
                    AsyncWebhookEventType::CategoryUpdated,
                    AsyncWebhookEventType::CategoryDeleted,
                    AsyncWebhookEventType::PageCreated,
                    AsyncWebhookEventType::PageUpdated,
                    AsyncWebhookEventType::PageDeleted,
                    AsyncWebhookEventType::CollectionCreated,
                    AsyncWebhookEventType::CollectionUpdated,
                    AsyncWebhookEventType::CollectionDeleted,
                ])
                .build(),
        )
        .build();
    debug!("Created AppManifest...");

    //Task queue
    let (sender, receiver) = tokio::sync::mpsc::channel(100);

    let app_state = AppState {
        task_queue_sender: sender,
        sitemap_config,
        manifest: app_manifest,
        config: config.clone(),
        target_channel: match dotenvy::var("CHANNEL_SLUG") {
            Ok(v) => v,
            Err(e) => {
                error!("Missing channel slug, Saleor will soon deprecate product queries without channel specified.");
                anyhow::bail!(e);
            }
        },
        saleor_app: Arc::new(Mutex::new(saleor_app)),
    };
    debug!("Created AppState...");

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
    info!("listening on {}", listener.local_addr()?);
    match axum::serve(listener, app).await {
        Ok(o) => Ok(o),
        Err(e) => anyhow::bail!(e),
    }
}
