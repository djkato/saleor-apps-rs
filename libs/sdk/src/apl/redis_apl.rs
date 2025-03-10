use async_trait::async_trait;
use std::time::Duration;

use redis::AsyncCommands;
#[cfg(feature = "tracing")]
use tracing::{debug, info};

use super::{AplError, APL};
use crate::AuthData;

#[derive(Debug, Clone)]
pub struct RedisApl {
    pub client: redis::Client,
    pub app_api_base_url: String,
}

#[async_trait]
impl APL for RedisApl {
    async fn get(&self, saleor_api_url: &str) -> Result<AuthData, AplError> {
        #[cfg(feature = "tracing")]
        debug!("get()");
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| AplError::Connection(e.to_string()))?;
        let val: String = conn
            .get(self.prepare_key(saleor_api_url))
            .await
            .map_err(|e| AplError::Connection(e.to_string()))?;
        let val: AuthData =
            serde_json::from_str(&val).map_err(|e| AplError::Serialization(e.to_string()))?;
        #[cfg(feature = "tracing")]
        info!("sucessful get");

        Ok(val)
    }
    async fn set(&self, auth_data: AuthData) -> Result<(), AplError> {
        #[cfg(feature = "tracing")]
        debug!("set()");
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| AplError::Connection(e.to_string()))?;
        conn.set::<_, _, String>(
            self.prepare_key(&auth_data.saleor_api_url),
            serde_json::to_string(&auth_data)
                .map_err(|e| AplError::Serialization(e.to_string()))?,
        )
        .await
        .map_err(|e| AplError::Connection(e.to_string()))?;
        #[cfg(feature = "tracing")]
        info!("sucessful set");
        Ok(())
    }
    async fn delete(&self, saleor_api_url: &str) -> Result<(), AplError> {
        #[cfg(feature = "tracing")]
        debug!("delete(), {}", saleor_api_url);
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| AplError::Connection(e.to_string()))?;
        let val: String = conn
            .get_del(self.prepare_key(saleor_api_url))
            .await
            .map_err(|e| AplError::Connection(e.to_string()))?;

        #[cfg(feature = "tracing")]
        debug!("sucessful delete(), {}", val);
        #[cfg(feature = "tracing")]
        info!("sucessful del");
        Ok(())
    }
    async fn is_ready(&self) -> Result<(), AplError> {
        #[cfg(feature = "tracing")]
        debug!("is_ready()");
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| AplError::Connection(e.to_string()))?;
        let val: String = redis::cmd("INFO")
            .arg("server")
            .query_async(&mut conn)
            .await
            .map_err(|e| AplError::Connection(e.to_string()))?;

        #[cfg(feature = "tracing")]
        debug!("sucessful is_ready(), info: {}", val);
        #[cfg(feature = "tracing")]
        info!("sucessful is_ready");
        Ok(())
    }
    async fn is_configured(&self) -> Result<(), AplError> {
        #[cfg(feature = "tracing")]
        debug!("is_configured()");
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| AplError::Connection(e.to_string()))?;
        let val: String = redis::cmd("INFO")
            .arg("server")
            .query_async(&mut conn)
            .await
            .map_err(|e| AplError::Connection(e.to_string()))?;

        #[cfg(feature = "tracing")]
        debug!("sucessful is_configured(), info: {}", val);
        #[cfg(feature = "tracing")]
        info!("sucessful is_configured");
        Ok(())
    }
    async fn get_all(&self) -> Result<Vec<AuthData>, AplError> {
        Err(AplError::NotSupported(
            "Redis doens't support getall".to_owned(),
        ))
    }
}

impl RedisApl {
    pub fn new(redis_url: &str, app_api_base_url: &str) -> Result<Self, AplError> {
        #[cfg(feature = "tracing")]
        debug!("creating redis apl with {redis_url}...");
        let client =
            redis::Client::open(redis_url).map_err(|e| AplError::Connection(e.to_string()))?;
        let mut conn = client
            .get_connection_with_timeout(Duration::from_secs(3))
            .map_err(|e| AplError::Connection(e.to_string()))?;
        let val: Result<String, redis::RedisError> =
            redis::cmd("INFO").arg("server").query(&mut conn);

        match val {
            Ok(_) => Ok(Self {
                client,
                app_api_base_url: app_api_base_url.to_owned(),
            }),
            Err(e) => Err(AplError::Connection(e.to_string())),
        }
    }
    pub fn prepare_key(&self, saleor_api_url: &str) -> String {
        let key = format!("{}:{saleor_api_url}", self.app_api_base_url);
        key
    }
}
