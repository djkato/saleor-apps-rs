use async_trait::async_trait;
use std::time::Duration;

use redis::{AsyncCommands, RedisError};
use tracing::{debug, info};

use super::APL;
use crate::AuthData;

#[derive(Debug, Clone)]
pub struct RedisApl {
    pub client: redis::Client,
    pub app_api_base_url: String,
}

#[derive(thiserror::Error, Debug)]
pub enum RedisAplError {
    #[error("Error during redis operation, {0}")]
    RedisError(#[from] RedisError),
    #[error("Failed parsing from/to json, {0}")]
    SerdeJsonDeError(#[from] serde_json::Error),
    #[error("RedisAPL doesn't support requested feature: {0}")]
    NotSupported(String),
}

#[async_trait]
impl APL<RedisAplError> for RedisApl {
    async fn get(&self, saleor_api_url: &str) -> Result<AuthData, RedisAplError> {
        debug!("get()");
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let val: String = conn.get(self.prepare_key(saleor_api_url)).await?;
        let val: AuthData = serde_json::from_str(&val)?;
        info!("sucessful get");

        Ok(val)
    }
    async fn set(&self, auth_data: AuthData) -> Result<(), RedisAplError> {
        debug!("set()");
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        conn.set::<_, _, String>(
            self.prepare_key(&auth_data.saleor_api_url),
            serde_json::to_string(&auth_data)?,
        )
        .await?;
        info!("sucessful set");
        Ok(())
    }
    async fn delete(&self, saleor_api_url: &str) -> Result<(), RedisAplError> {
        debug!("delete(), {}", saleor_api_url);
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let val: String = conn.get_del(self.prepare_key(saleor_api_url)).await?;

        debug!("sucessful delete(), {}", val);
        info!("sucessful del");
        Ok(())
    }
    async fn is_ready(&self) -> Result<(), RedisAplError> {
        debug!("is_ready()");
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let val: String = redis::cmd("INFO")
            .arg("server")
            .query_async(&mut conn)
            .await?;

        debug!("sucessful is_ready(), info: {}", val);
        info!("sucessful is_ready");
        Ok(())
    }
    async fn is_configured(&self) -> Result<(), RedisAplError> {
        debug!("is_configured()");
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let val: String = redis::cmd("INFO")
            .arg("server")
            .query_async(&mut conn)
            .await?;

        debug!("sucessful is_configured(), info: {}", val);
        info!("sucessful is_configured");
        Ok(())
    }
    async fn get_all(&self) -> Result<Vec<AuthData>, RedisAplError> {
        Err(RedisAplError::NotSupported(
            "Redis doens't support getall".to_owned(),
        ))
    }
}

impl RedisApl {
    pub fn new(redis_url: &str, app_api_base_url: &str) -> Result<Self, RedisAplError> {
        debug!("creating redis apl with {redis_url}...");
        let client = redis::Client::open(redis_url)?;
        let mut conn = client.get_connection_with_timeout(Duration::from_secs(3))?;
        let val: Result<String, redis::RedisError> =
            redis::cmd("INFO").arg("server").query(&mut conn);

        match val {
            Ok(_) => Ok(Self {
                client,
                app_api_base_url: app_api_base_url.to_owned(),
            }),
            Err(e) => Err(e.into()),
        }
    }
    pub fn prepare_key(&self, saleor_api_url: &str) -> String {
        let key = format!("{}:{saleor_api_url}", self.app_api_base_url);
        key
    }
}
