#[cfg(feature = "file_apl")]
pub mod file_apl;
#[cfg(feature = "redis_apl")]
pub mod redis_apl;

use crate::AuthData;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AplType {
    Redis,
    File,
}

#[async_trait]
pub trait APL<E>: Send + Sync + std::fmt::Debug {
    async fn get(&self, saleor_api_url: &str) -> Result<AuthData, E>;
    async fn set(&self, auth_data: AuthData) -> Result<(), E>;
    async fn delete(&self, saleor_api_url: &str) -> Result<(), E>;
    async fn get_all(&self) -> Result<Vec<AuthData>, E>;
    async fn is_ready(&self) -> Result<(), E>;
    async fn is_configured(&self) -> Result<(), E>;
}
