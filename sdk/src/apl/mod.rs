#[cfg(feature = "file_apl")]
pub mod file_apl;
#[cfg(feature = "redis_apl")]
pub mod redis_apl;

use crate::AuthData;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AplType {
    Redis,
    File,
}

#[async_trait]
pub trait APL: Send + Sync + std::fmt::Debug {
    async fn get(&self, saleor_api_url: &str) -> Result<AuthData>;
    async fn set(&self, auth_data: AuthData) -> Result<()>;
    async fn delete(&self, saleor_api_url: &str) -> Result<()>;
    async fn get_all(&self) -> Result<Vec<AuthData>>;
    async fn is_ready(&self) -> Result<()>;
    async fn is_configured(&self) -> Result<()>;
}
