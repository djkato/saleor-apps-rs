mod app;
mod queries;
mod routes;

use anyhow::Context;
use fd_lock::RwLock;
use saleor_app_sdk::{
    config::Config,
    manifest::{AppManifest, AppPermission},
    webhooks::{AsyncWebhookEventType, WebhookManifest},
    SaleorApp,
};
use std::{fs::File, sync::Arc};
use tokio::sync::Mutex;
use tracing::debug;

use crate::{
    app::{trace_to_std, AppState, SitemapConfig, XmlCache},
    queries::event_subjects_updated::EVENTS_QUERY,
    routes::create_routes,
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

    debug!("{}/sitemap_index.xml.gz", sitemap_config.target_folder);
    let app_state = AppState {
        sitemap_file_index: Arc::new(RwLock::new(File::options().write(true).create(true).open(
            format!("{}/sitemap_index.xml", sitemap_config.target_folder),
        )?)),
        sitemap_file_products: vec![],
        sitemap_file_categories: vec![],
        sitemap_file_collections: vec![],
        sitemap_file_pages: vec![],
        sitemap_config,
        xml_cache: XmlCache::new(&config.apl_url, &config.app_api_base_url)?,
        manifest: app_manifest,
        config: config.clone(),
        saleor_app: Arc::new(Mutex::new(saleor_app)),
    };
    debug!("Created AppState...");
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
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    match axum::serve(listener, app).await {
        Ok(o) => Ok(o),
        Err(e) => anyhow::bail!(e),
    }
}
