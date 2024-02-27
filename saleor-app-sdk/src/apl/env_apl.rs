use super::APL;
use anyhow::Result;

#[derive(Clone, Debug)]
/**
is not implemented yet!
*/
pub struct EnvApl {}

impl APL for EnvApl {
    async fn set(&self, auth_data: crate::AuthData) -> Result<()> {
        todo!()
    }
    async fn get(&self, saleor_api_url: &str) -> Result<crate::AuthData> {
        todo!()
    }
    async fn get_all(&self) -> Result<Vec<crate::AuthData>> {
        todo!()
    }
    async fn delete(&self, saleor_api_url: &str) -> Result<()> {
        todo!()
    }
    async fn is_ready(&self) -> Result<()> {
        todo!()
    }
    async fn is_configured(&self) -> Result<()> {
        todo!()
    }
}
