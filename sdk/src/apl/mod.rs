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

#[derive(thiserror::Error, Debug)]
pub enum AplError {
    #[error("APL encountered a Connection Error: {0}")]
    Connection(String),
    #[error("APL encountered a filesystem IO error: {0}")]
    IO(String),
    #[error("APL Error happened during De/Serialization: {0}")]
    Serialization(String),
    #[error("Requested APL doesn't support requested feature: {0}")]
    NotSupported(String),
    #[error("Key or value wasn't found during Get")]
    NotFound(String),
}

#[async_trait]
pub trait APL: Send + Sync + std::fmt::Debug {
    async fn get(&self, saleor_api_url: &str) -> Result<AuthData, AplError>;
    async fn set(&self, auth_data: AuthData) -> Result<(), AplError>;
    async fn delete(&self, saleor_api_url: &str) -> Result<(), AplError>;
    async fn get_all(&self) -> Result<Vec<AuthData>, AplError>;
    async fn is_ready(&self) -> Result<(), AplError>;
    async fn is_configured(&self) -> Result<(), AplError>;
}
