

use super::APL;
use anyhow::{Result};
use async_trait::async_trait;


#[derive(Clone, Debug)]
/**
is not implemented yet!
*/
pub struct FileApl {
    pub path: String,
}

#[async_trait]
impl APL for FileApl {
    async fn set(&self, _auth_data: crate::AuthData) -> Result<()> {
        todo!()
    }
    async fn get(&self, _saleor_api_url: &str) -> Result<crate::AuthData> {
        todo!()
    }
    async fn get_all(&self) -> Result<Vec<crate::AuthData>> {
        todo!()
    }
    async fn delete(&self, _saleor_api_url: &str) -> Result<()> {
        todo!()
    }
    async fn is_ready(&self) -> Result<()> {
        todo!()
    }
    async fn is_configured(&self) -> Result<()> {
        todo!()
    }
}
