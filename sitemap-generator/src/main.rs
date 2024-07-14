#![allow(
    non_upper_case_globals,
    clippy::large_enum_variant,
    clippy::upper_case_acronyms,
    dead_code
)]
#![feature(let_chains)]
// #![deny(clippy::unwrap_used, clippy::expect_used)]
mod app;
mod queries;
mod routes;
mod sitemap;

#[cfg(test)]
mod tests;

use axum::Router;
use saleor_app_sdk::{
    config::Config,
    manifest::{cargo_info, AppManifestBuilder, AppPermission},
    webhooks::{AsyncWebhookEventType, WebhookManifestBuilder},
    SaleorApp,
};
use sitemap::event_handler::EventHandler;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info};

use crate::{
    app::{trace_to_std, AppState, SitemapConfig},
    queries::event_subjects_updated::EVENTS_QUERY,
    routes::create_routes,
};

#[tokio::main]
async fn main() {
    debug!("Creating configs...");
    let config = Config::load().unwrap();
    trace_to_std(&config).unwrap();
    let sitemap_config = SitemapConfig::load().unwrap();

    let app = create_app(&config, sitemap_config).await;

    let listener = tokio::net::TcpListener::bind(
        "0.0.0.0:".to_owned()
            + config
                .app_api_base_url
                .split(':')
                .collect::<Vec<_>>()
                .get(2)
                .unwrap_or(&"3000"),
    )
    .await
    .unwrap();
    info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn create_app(config: &Config, sitemap_config: SitemapConfig) -> Router {
    let saleor_app = SaleorApp::new(config).unwrap();

    debug!("Creating saleor App...");
    let app_manifest = AppManifestBuilder::new(config, cargo_info!())
        .add_permissions(vec![
            AppPermission::ManageProducts,
            AppPermission::ManagePages,
        ])
        .add_webhook(
            WebhookManifestBuilder::new(config)
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

    let (sender, receiver) = tokio::sync::mpsc::channel(100);

    EventHandler::start(sitemap_config.clone(), receiver);

    let app_state = AppState {
        task_queue_sender: sender,
        sitemap_config,
        manifest: app_manifest,
        config: config.clone(),
        target_channel: match dotenvy::var("CHANNEL_SLUG") {
            Ok(v) => v,
            Err(_) => {
                error!("Missing channel slug, Saleor will soon deprecate product queries without channel specified.");
                "".to_string()
            }
        },
        saleor_app: Arc::new(Mutex::new(saleor_app)),
    };
    debug!("Created AppState...");
    create_routes(app_state)
}
