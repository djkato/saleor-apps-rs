use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use crate::AuthData;

use super::APL;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Clone, Debug)]
/**
 Only works for this app, can't have multiple apps use same file
*/
pub struct FileApl {
    pub path: String,
}

#[async_trait]
impl APL for FileApl {
    async fn set(&self, auth_data: crate::AuthData) -> Result<()> {
        let path = std::path::Path::new(&self.path);
        debug!("reading from {:?}", &path);
        let mut auths: FileStructure = serde_json::from_str(&std::fs::read_to_string(path)?)?;

        auths.insert(auth_data.saleor_api_url.clone(), auth_data);

        debug!("writing to {:?}", &path);
        std::fs::write(path, &serde_json::to_string_pretty(&auths)?.as_bytes())?;
        Ok(())
    }

    async fn get(&self, saleor_api_url: &str) -> Result<crate::AuthData> {
        let path = std::path::Path::new(&self.path);
        debug!("reading from {:?}", &path);
        let auth_data: FileStructure = serde_json::from_str(&std::fs::read_to_string(path)?)?;
        auth_data
            .get(saleor_api_url)
            .cloned()
            .ok_or(anyhow!("AuthData for {saleor_api_url} not found"))
    }

    async fn get_all(&self) -> Result<Vec<crate::AuthData>> {
        let path = std::path::Path::new(&self.path);
        debug!("reading from {:?}", &path);
        let auth_data: FileStructure = serde_json::from_str(&std::fs::read_to_string(path)?)?;
        Ok(auth_data.0.values().cloned().collect())
    }

    async fn delete(&self, saleor_api_url: &str) -> Result<()> {
        let path = std::path::Path::new(&self.path);
        debug!("reading from {:?}", &path);
        let mut auths: FileStructure = serde_json::from_str(&std::fs::read_to_string(path)?)?;
        auths.remove(saleor_api_url);

        debug!("writing to {:?}", &path);
        std::fs::write(path, &serde_json::to_string_pretty(&auths)?.as_bytes())?;
        Ok(())
    }

    async fn is_ready(&self) -> Result<()> {
        Ok(())
    }

    async fn is_configured(&self) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct FileStructure(HashMap<String, AuthData>);

impl Deref for FileStructure {
    type Target = HashMap<String, AuthData>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for FileStructure {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
