use anyhow::bail;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, FixedOffset};
use std::{sync::Arc, time::Duration};
use tracing_subscriber::EnvFilter;

use redis::{AsyncCommands, Client};
use saleor_app_sdk::{config::Config, manifest::AppManifest, SaleorApp};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, level_filters::LevelFilter};
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

pub fn trace_to_std(config: &Config) {
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::DEBUG.into())
        .from_env()
        .unwrap()
        .add_directive(
            format!("{}={}", env!("CARGO_PKG_NAME"), config.log_level)
                .parse()
                .unwrap(),
        );
    tracing_subscriber::fmt()
        .with_max_level(config.log_level)
        .with_env_filter(filter)
        .with_target(true)
        .compact()
        .init();
}

/**
 * Sitemaps have a limit of 10mb, so we create an index and split all paths between multiple
 * sitemaps.
 */
#[derive(Debug, Clone)]
pub struct AppState {
    pub xml_cache: Arc<tokio::sync::Mutex<XmlCache>>,
    pub saleor_app: Arc<tokio::sync::Mutex<SaleorApp>>,
    pub config: Config,
    pub sitemap_config: SitemapConfig,
    pub manifest: AppManifest,
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
}

impl SitemapConfig {
    pub fn load() -> Result<Self, envy::Error> {
        dotenvy::dotenv().unwrap();
        
        envy::from_env::<SitemapConfig>()
    }
}

#[derive(Debug)]
pub struct XmlCache {
    client: Client,
    app_api_base_url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct XmlData {
    pub id: cynic::Id,
    pub slug: String,
    pub relations: Vec<cynic::Id>,
    pub data_type: XmlDataType,
    pub last_modified: DateTime<FixedOffset>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum XmlDataType {
    Category,
    Product,
    Page,
    Collection,
}

impl XmlCache {
    pub fn new(redis_url: &str, app_api_base_url: &str) -> anyhow::Result<Self> {
        debug!("creating XmlCache...");
        let client = redis::Client::open(redis_url)?;
        let mut conn = client.get_connection_with_timeout(Duration::from_secs(3))?;
        let val: Result<String, redis::RedisError> =
            redis::cmd("INFO").arg("server").query(&mut conn);

        match val {
            Ok(_) => Ok(Self {
                client,
                app_api_base_url: app_api_base_url.to_owned(),
            }),
            Err(e) => bail!("failed redis connection(XmlCache), {:?}", e),
        }
    }

    /**
     * ONLY USE IF YOU KNOW WHAT YOU'RE DOING! Will flush entire cache, run regenerate() from
     * webhooks to renew.
     */
    pub async fn delete_all(&self, saleor_api_url: &str) -> anyhow::Result<()> {
        debug!("xml data delete_cache()");
        let mut conn = self.client.get_async_connection().await?;
        conn.del(self.prepare_key(saleor_api_url)).await?;

        info!("sucessful cache wipe");
        Ok(())
    }

    pub async fn get_all(&self, saleor_api_url: &str) -> anyhow::Result<Vec<XmlData>> {
        debug!("xml data get_all()");
        let mut conn = self.client.get_async_connection().await?;
        let res: Vec<u8> = conn.get(self.prepare_key(saleor_api_url)).await?;
        let cache: Vec<XmlData> = serde_cbor::from_slice(&res)?;

        info!("sucessful cache get");

        Ok(cache)
    }

    pub async fn set(&self, data: Vec<XmlData>, saleor_api_url: &str) -> anyhow::Result<()> {
        debug!("xml data set()");
        let mut conn = self.client.get_async_connection().await?;
        conn.set(self.prepare_key(saleor_api_url), serde_cbor::to_vec(&data)?)
            .await?;
        info!("sucessful cache set");
        Ok(())
    }

    pub fn prepare_key(&self, saleor_api_url: &str) -> String {
        let key = format!("{}:{saleor_api_url}", self.app_api_base_url);
        key
    }
}
