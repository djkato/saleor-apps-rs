use crate::AuthData;

use super::queries::{
    DeleteAppMetadata, DeleteAppMetadataVariables, GetAppMetadata, GetAppMetadataVariables,
    MetadataInput, SetAppMetadata, SetAppMetadataVariables,
};
use super::SettingsManager;
use async_trait::async_trait;
use cynic::{http::SurfExt, QueryBuilder};
use cynic::{GraphQlError, MutationBuilder};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::hash::Hash;
use std::str::FromStr;
use tracing::debug;

#[derive(Debug, Clone)]
pub struct Metadata<K: Hash + Eq + FromStr + ToString, V: Serialize + DeserializeOwned>(
    pub HashMap<K, V>,
);

#[derive(Debug, Clone)]
pub struct MetadataSettingsManager<
    K: Hash + Eq + FromStr + ToString,
    V: Serialize + DeserializeOwned,
> {
    pub metadata: Metadata<K, V>,
    pub auth_data: AuthData,
}

#[derive(thiserror::Error, Debug)]
pub enum MetadataSettingsManagerError {
    #[error("Error during graphql querys http request, {0}")]
    HttpRequestError(surf::Error),
    #[error("Graphql query contains errors, {0}")]
    GraphQlError(#[from] GraphQlError),
    #[error("Key was not found in hashmap/metadata")]
    KeyNotFound,
}

#[async_trait]
impl<
        K: Hash + Eq + Send + Sync + FromStr + ToString,
        V: Send + Sync + Clone + Serialize + DeserializeOwned,
    > SettingsManager<K, V, MetadataSettingsManagerError> for MetadataSettingsManager<K, V>
{
    async fn get(&mut self, key: K, domain: &str) -> Result<V, MetadataSettingsManagerError> {
        //TODO: Create a cache instead of refetching every time
        self.remote_get(Some(domain)).await?;
        let val = self
            .metadata
            .0
            .get(&key)
            .ok_or(MetadataSettingsManagerError::KeyNotFound)?;
        Ok(val.clone())
    }

    async fn set(
        &mut self,
        key: K,
        value: V,
        domain: &str,
    ) -> Result<(), MetadataSettingsManagerError> {
        let _ = self.metadata.0.insert(key, value);
        self.remote_set(Some(domain)).await?;
        Ok(())
    }

    async fn delete(&mut self, key: K, domain: &str) -> Result<V, MetadataSettingsManagerError> {
        let removed = self
            .metadata
            .0
            .remove(&key)
            .ok_or(MetadataSettingsManagerError::KeyNotFound)?;
        self.remote_delete(Some(domain), vec![key]).await?;
        Ok(removed)
    }
}

impl<
        K: Hash + Eq + Send + Sync + FromStr + ToString,
        V: Send + Sync + DeserializeOwned + Serialize,
    > MetadataSettingsManager<K, V>
{
    pub fn to_metadata_vec(&self) -> Vec<MetadataInput> {
        self.metadata
            .0
            .iter()
            .map(|(k, v)| MetadataInput {
                key: k.to_string(),
                value: serde_json::to_string(v)
                    .expect("failed parsing metadatainput to json string"),
            })
            .collect::<Vec<_>>()
    }

    pub async fn remote_delete(
        &mut self,
        api_url: Option<&str>,
        keys: Vec<K>,
    ) -> Result<(), MetadataSettingsManagerError> {
        let app_id = cynic::Id::new(&self.auth_data.app_id);
        let keys = keys.into_iter().map(|k| k.to_string()).collect::<Vec<_>>();
        let operation = DeleteAppMetadata::build(DeleteAppMetadataVariables {
            app_id: &app_id,
            keys: keys.iter().map(|k| k.as_str()).collect(),
        });
        match surf::post(api_url.unwrap_or(&self.auth_data.saleor_api_url))
            .header("authorization-bearer", &self.auth_data.token)
            .run_graphql(operation)
            .await
        {
            Ok(response) => {
                if let Some(res_errors) = response.errors {
                    if let Some(e) = res_errors.first().cloned() {
                        return Err(e.into());
                    }
                }
                Ok(())
            }
            Err(e) => {
                debug!("{:?}", e);
                Err(MetadataSettingsManagerError::HttpRequestError(e))
            }
        }
    }

    /**
    sets metadata (not the private ones) in saleor app
    */
    pub async fn remote_set(
        &mut self,
        api_url: Option<&str>,
    ) -> Result<(), MetadataSettingsManagerError> {
        let app_id = cynic::Id::new(&self.auth_data.app_id);
        let operation = SetAppMetadata::build(SetAppMetadataVariables {
            app_id: &app_id,
            input: self.to_metadata_vec(),
        });
        match surf::post(api_url.unwrap_or(&self.auth_data.saleor_api_url))
            .header("authorization-bearer", &self.auth_data.token)
            .run_graphql(operation)
            .await
        {
            Ok(response) => {
                if let Some(res_errors) = response.errors {
                    if let Some(e) = res_errors.first().cloned() {
                        return Err(e.into());
                    }
                }
                Ok(())
            }
            Err(e) => {
                debug!("{:?}", e);
                Err(MetadataSettingsManagerError::HttpRequestError(e))
            }
        }
    }

    /**
    refetches metadata from saleor (not the private ones) and parses them to a HashMap<K,V>
    */
    pub async fn remote_get(
        &mut self,
        api_url: Option<&str>,
    ) -> Result<&mut Self, MetadataSettingsManagerError> {
        let app_id = cynic::Id::new(&self.auth_data.app_id);
        let operation = GetAppMetadata::build(GetAppMetadataVariables { app_id: &app_id });
        match surf::post(api_url.unwrap_or(&self.auth_data.saleor_api_url))
            .header("authorization-bearer", &self.auth_data.token)
            .run_graphql(operation)
            .await
        {
            Ok(response) => {
                if let Some(res_errors) = response.errors {
                    if let Some(e) = res_errors.first().cloned() {
                        return Err(e.into());
                    }
                }
                if let Some(res) = response.data {
                    if let Some(app) = res.app {
                        let hashmap = app
                            .metadata
                            .into_iter()
                            .filter_map(|m| {
                                K::from_str(&m.key)
                                    .ok()
                                    .zip(serde_json::from_str(&m.value).ok())
                            })
                            .collect::<HashMap<K, V>>();
                        self.metadata = Metadata(hashmap);
                    }
                }
                self.metadata = Metadata(HashMap::new());
                Ok(self)
            }
            Err(e) => {
                debug!("{:?}", e);
                Err(MetadataSettingsManagerError::HttpRequestError(e))
            }
        }
    }

    /**
    creates a new manager, and also prefetches metadata from saleor (not the private ones) and parses them to a HashMap<K,V>
    Make sure your types convert correctly into these types through FromStr, if either key or value is not converted correctly, both get dropped silently!
    */
    pub async fn new(auth_data: AuthData) -> Result<Self, MetadataSettingsManagerError> {
        let mut mngr = Self {
            auth_data,
            metadata: Metadata(HashMap::new()),
        };
        mngr.remote_get(None).await?;
        Ok(mngr)
    }
}
