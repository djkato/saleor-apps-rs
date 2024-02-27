use std::time::Duration;

use redis::AsyncCommands;
use tracing::{debug, info};

use super::APL;
use crate::AuthData;
use anyhow::{bail, Result};

#[derive(Debug, Clone)]
pub struct RedisApl {
    pub client: redis::Client,
    pub app_api_base_url: String,
}

impl APL for RedisApl {
    async fn get(&self, saleor_api_url: &str) -> Result<AuthData> {
        debug!(" get()");
        let mut conn = self.client.get_async_connection().await?;
        let val: String = conn.get(self.prepare_key(saleor_api_url)).await?;
        debug!("received {val}");
        let val: AuthData = serde_json::from_str(&val)?;
        info!("sucessful get");
        debug!("parsed {val}");

        Ok(val)
    }
    async fn set(&self, auth_data: AuthData) -> Result<()> {
        debug!("set(), {}", auth_data);
        let mut conn = self.client.get_async_connection().await?;
        conn.set(
            self.prepare_key(&auth_data.saleor_api_url),
            serde_json::to_string(&auth_data)?,
        )
        .await?;
        info!("sucessful set");
        Ok(())
    }
    async fn delete(&self, saleor_api_url: &str) -> Result<()> {
        debug!("delete(), {}", saleor_api_url);
        let mut conn = self.client.get_async_connection().await?;
        let val: String = conn.get_del(self.prepare_key(saleor_api_url)).await?;

        debug!("sucessful delete(), {}", val);
        info!("sucessful del");
        Ok(())
    }
    async fn is_ready(&self) -> Result<()> {
        debug!("is_ready()");
        let mut conn = self.client.get_async_connection().await?;
        let val: String = redis::cmd("INFO")
            .arg("server")
            .query_async(&mut conn)
            .await?;

        debug!("sucessful is_ready(), info: {}", val);
        info!("sucessful is_ready");
        Ok(())
    }
    async fn is_configured(&self) -> Result<()> {
        debug!("is_configured()");
        let mut conn = self.client.get_async_connection().await?;
        let val: String = redis::cmd("INFO")
            .arg("server")
            .query_async(&mut conn)
            .await?;

        debug!("sucessful is_configured(), info: {}", val);
        info!("sucessful is_configured");
        Ok(())
    }
    async fn get_all(&self) -> Result<Vec<AuthData>> {
        anyhow::bail!("Redis doens't support getall")
    }
}

impl RedisApl {
    pub fn new(redis_url: String, app_api_base_url: String) -> Result<Self> {
        let client = redis::Client::open(redis_url)?;
        let mut conn = client.get_connection_with_timeout(Duration::from_secs(3))?;
        let val: Result<String, redis::RedisError> =
            redis::cmd("INFO").arg("server").query(&mut conn);

        match val {
            Ok(_) => Ok(Self {
                client,
                app_api_base_url,
            }),
            Err(e) => bail!("failed redis connection, {:?}", e),
        }
    }
    pub fn prepare_key(&self, saleor_api_url: &str) -> String {
        let key = format!("{}:{saleor_api_url}", self.app_api_base_url);
        debug!("made key:'{}'", key);
        key
    }
}
