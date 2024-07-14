use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tracing_subscriber::EnvFilter;

use saleor_app_sdk::{config::Config, manifest::AppManifest, SaleorApp};
use serde::{Deserialize, Serialize};
use tracing::level_filters::LevelFilter;

use crate::sitemap::event_handler::Event;

// Make our own error that wraps `anyhow::Error`.
pub struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

pub fn trace_to_std(config: &Config) -> anyhow::Result<()> {
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env()?
        .add_directive(format!("{}={}", env!("CARGO_PKG_NAME"), config.log_level).parse()?);
    tracing_subscriber::fmt()
        .with_max_level(config.log_level)
        .with_env_filter(filter)
        .with_target(true)
        .compact()
        .init();
    Ok(())
}

/**
 * Sitemaps have a limit of 10mb, so we create an index and split all paths between multiple
 * sitemaps.
 */
#[derive(Debug, Clone)]
pub struct AppState {
    pub saleor_app: Arc<tokio::sync::Mutex<SaleorApp>>,
    pub config: Config,
    pub target_channel: String,
    pub sitemap_config: SitemapConfig,
    pub manifest: AppManifest,
    pub task_queue_sender: Sender<Event>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SitemapConfig {
    #[serde(rename = "sitemap_target_folder")]
    pub target_folder: String,
    #[serde(rename = "sitemap_product_template")]
    pub product_template: String,
    #[serde(rename = "sitemap_category_template")]
    pub category_template: String,
    #[serde(rename = "sitemap_pages_template")]
    pub pages_template: String,
    #[serde(rename = "sitemap_collection_template")]
    pub collection_template: String,
    #[serde(rename = "sitemap_index_hostname")]
    pub index_hostname: String,
    #[serde(rename = "sitemap_allowed_host")]
    pub allowed_host: String,
}

impl SitemapConfig {
    pub fn load() -> Result<Self, envy::Error> {
        _ = dotenvy::dotenv();
        envy::from_env::<SitemapConfig>()
    }
}
