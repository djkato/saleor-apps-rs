use std::path::Path;

use super::APL;
use anyhow::{bail, Result};
use std::fs::{read, write};

#[derive(Clone, Debug)]
/**
is not implemented yet!
*/
pub struct FileApl {
    pub path: String,
}

impl APL for FileApl {
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
