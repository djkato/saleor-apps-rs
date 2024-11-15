pub mod encrypted_metadata;
pub mod metadata;
pub mod queries;

use async_trait::async_trait;

#[async_trait]
pub trait SettingsManager<K, V, E>: Send + Sync {
    async fn get(&mut self, key: K, domain: &str) -> Result<V, E>;
    async fn set(&mut self, key: K, value: V, domain: &str) -> Result<(), E>;
    async fn delete(&mut self, key: K, domain: &str) -> Result<V, E>;
}
