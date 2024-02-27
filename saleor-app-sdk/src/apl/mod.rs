pub mod env_apl;
pub mod file_apl;
pub mod redis_apl;

use crate::AuthData;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::future::Future;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AplType {
    Redis,
    File,
    Env,
}

pub trait APL: Sized + Send + Sync + Clone + std::fmt::Debug {
    fn get(&self, saleor_api_url: &str) -> impl Future<Output = Result<AuthData>> + Send;
    fn set(&self, auth_data: AuthData) -> impl Future<Output = Result<()>> + Send;
    fn delete(&self, saleor_api_url: &str) -> impl Future<Output = Result<()>> + Send;
    fn get_all(&self) -> impl Future<Output = Result<Vec<AuthData>>> + Send;
    fn is_ready(&self) -> impl Future<Output = Result<()>> + Send;
    fn is_configured(&self) -> impl Future<Output = Result<()>> + Send;
}
