use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use crate::AuthData;

use super::{AplError, APL};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
#[cfg(feature = "tracing")]
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
    async fn set(&self, auth_data: crate::AuthData) -> Result<(), AplError> {
        let path = std::path::Path::new(&self.path);
#[cfg(feature = "tracing")]
        debug!("reading from {:?}", &path);
        let mut auths: FileStructure;
        match path.is_file() {
            true => {
                auths = serde_json::from_str(
                    &std::fs::read_to_string(path).map_err(|e| AplError::IO(e.to_string()))?,
                )
                .map_err(|e| AplError::Serialization(e.to_string()))?
            }
            false => auths = FileStructure(HashMap::new()),
        }

        auths.insert(auth_data.saleor_api_url.clone(), auth_data);

#[cfg(feature = "tracing")]
        debug!("writing to {:?}", &path);
        std::fs::write(
            path,
            serde_json::to_string_pretty(&auths)
                .map_err(|e| AplError::Serialization(e.to_string()))?
                .as_bytes(),
        )
        .map_err(|e| AplError::IO(e.to_string()))?;
        Ok(())
    }

    async fn get(&self, saleor_api_url: &str) -> Result<crate::AuthData, AplError> {
        let path = std::path::Path::new(&self.path);
#[cfg(feature = "tracing")]
        debug!("reading from {:?}", &path);
        let auth_data: FileStructure = serde_json::from_str(
            &std::fs::read_to_string(path).map_err(|e| AplError::IO(e.to_string()))?,
        )
        .map_err(|e| AplError::Serialization(e.to_string()))?;
        auth_data
            .get(saleor_api_url)
            .cloned()
            .ok_or(AplError::NotFound(
                "haven't found entry for given url".to_owned(),
            ))
    }

    async fn get_all(&self) -> Result<Vec<crate::AuthData>, AplError> {
        let path = std::path::Path::new(&self.path);
#[cfg(feature = "tracing")]
        debug!("reading from {:?}", &path);
        let auth_data: FileStructure = serde_json::from_str(
            &std::fs::read_to_string(path).map_err(|e| AplError::IO(e.to_string()))?,
        )
        .map_err(|e| AplError::Serialization(e.to_string()))?;
        Ok(auth_data.0.values().cloned().collect())
    }

    async fn delete(&self, saleor_api_url: &str) -> Result<(), AplError> {
        let path = std::path::Path::new(&self.path);
#[cfg(feature = "tracing")]
        debug!("reading from {:?}", &path);
        let mut auths: FileStructure = serde_json::from_str(
            &std::fs::read_to_string(path).map_err(|e| AplError::IO(e.to_string()))?,
        )
        .map_err(|e| AplError::Serialization(e.to_string()))?;
        auths.remove(saleor_api_url);

#[cfg(feature = "tracing")]
        debug!("writing to {:?}", &path);
        std::fs::write(
            path,
            serde_json::to_string_pretty(&auths)
                .map_err(|e| AplError::Serialization(e.to_string()))?
                .as_bytes(),
        )
        .map_err(|e| AplError::IO(e.to_string()))?;
        Ok(())
    }

    async fn is_ready(&self) -> Result<(), AplError> {
        Ok(())
    }

    async fn is_configured(&self) -> Result<(), AplError> {
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
