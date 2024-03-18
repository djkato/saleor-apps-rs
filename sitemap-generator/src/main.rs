#![feature(let_chains)]
#![deny(clippy::unwrap_used, clippy::expect_used)]
mod app;
mod queries;
mod routes;

use anyhow::Context;
use saleor_app_sdk::{
    config::Config,
    manifest::{AppManifest, AppPermission},
    webhooks::{AsyncWebhookEventType, WebhookManifest},
    SaleorApp,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};

use crate::{
    app::{trace_to_std, AppState, SitemapConfig, XmlCache},
    queries::event_subjects_updated::EVENTS_QUERY,
    routes::{create_routes, register::regenerate},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::load()?;
    trace_to_std(&config);
    let sitemap_config = SitemapConfig::load()?;
    debug!("Creating configs...");

    let saleor_app = SaleorApp::new(&config)?;

    debug!("Creating saleor App...");

    let app_manifest = AppManifest::new(&config)
        .add_permissions(vec![
            AppPermission::ManageProducts,
            AppPermission::ManagePages,
        ])
        .add_webhook(
            WebhookManifest::new(&config)
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
    let app_state = AppState {
        sitemap_config,
        xml_cache: Arc::new(Mutex::new(XmlCache::new(
            &config.apl_url,
            &config.app_api_base_url,
        )?)),
        manifest: app_manifest,
        config: config.clone(),
        saleor_app: Arc::new(Mutex::new(saleor_app)),
    };
    debug!("Created AppState...");

    {
        // either clear the cache, regenerate or both from command args
        let mut pargs = pico_args::Arguments::from_env();

        if let Some(for_url) = pargs.opt_value_from_str::<_, String>("--for-url")? {
            if pargs.contains("--cache-clear") {
                let xml_cache = app_state.xml_cache.lock().await;
                xml_cache.delete_all(&for_url).await?;
                debug!("Cleared Xml Cache for {for_url}");
            }

            if pargs.contains("--cache-regenerate") {
                regenerate(app_state.clone(), for_url).await?;
            }
            std::process::exit(0)
        }
    }

    let app = create_routes(app_state);
    let listener = tokio::net::TcpListener::bind(
        &config
            .app_api_base_url
            .split("//")
            .collect::<Vec<_>>()
            .get(1)
            .context("APP_API_BASE_URL invalid format")?,
    )
    .await?;
    info!("listening on {}", listener.local_addr().unwrap());
    match axum::serve(listener, app).await {
        Ok(o) => Ok(o),
        Err(e) => anyhow::bail!(e),
    }
}
