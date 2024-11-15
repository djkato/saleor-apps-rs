pub mod apl;
#[cfg(feature = "bridge")]
pub mod bridge;
pub mod config;
pub mod headers;
pub mod manifest;
#[cfg(feature = "middleware")]
pub mod middleware;
#[cfg(feature = "settings_manager")]
pub mod settings_manager;
pub mod webhooks;

#[cfg(feature = "redis_apl")]
use apl::redis_apl::RedisAplError;
use apl::{AplType, APL};
use config::Config;
use serde::{Deserialize, Serialize};

#[cfg(feature = "file_apl")]
use crate::apl::file_apl::FileApl;
#[cfg(feature = "redis_apl")]
use crate::apl::redis_apl::RedisApl;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub auth_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthData {
    pub domain: Option<String>,
    pub token: String,
    pub saleor_api_url: String,
    pub app_id: String,
    pub jwks: Option<String>,
}

impl std::fmt::Display for AuthData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(domain:{}\ntoken:{}\nsaleor_api_url:{}\napp_id:{}\njwks:{})",
            self.domain.clone().unwrap_or_default(),
            self.token,
            self.saleor_api_url,
            self.app_id,
            self.jwks.clone().unwrap_or_default()
        )
    }
}

#[derive(Debug)]
pub struct SaleorApp<E> {
    pub apl: Box<dyn APL<E>>,
}

#[derive(thiserror::Error, Debug)]
pub enum CreateSaleorAppError {
    #[error("Feature needed to use this APL is not enabled in cargo.toml")]
    MissingFeature(String),
    #[cfg(feature = "redis_apl")]
    #[error("failed creating redis_apl, {0}")]
    #[cfg(feature = "redis_apl")]
    RedisAplError(#[from] RedisAplError),
}

impl<E: std::error::Error> SaleorApp<E> {
    pub fn new(config: &Config) -> Result<SaleorApp<E>, CreateSaleorAppError> {
        use AplType::{File, Redis};
        fn decide_apl<E>(config: &Config) -> Box<dyn APL<E>> {
            match config.apl {
                Redis => {
                    #[cfg(feature = "redis_apl")]
                    return Box::new(
                        RedisApl::new(&config.apl_url, &config.app_api_base_url)
                            .expect("failed creating redisapl"),
                    );

                    #[cfg(not(feature = "redis_apl"))]
                    {
                        return CreateSaleorAppError ::MissingFeature("Tried starting app with redis apl that wasn't present at compile time (cargo feature missing)");
                    }
                }
                File => {
                    #[cfg(feature = "file_apl")]
                    return Box::new(FileApl {
                        path: config.apl_url.to_owned(),
                    });
                    #[cfg(not(feature = "file_apl"))]
                    {
                        return CreateSaleorAppError ::MissingFeature("Tried starting app with file apl that wasn't present at compile time (cargo feature missing)");
                    }
                }
            }
        }
        let apl = decide_apl(config);
        Ok(SaleorApp { apl })
    }
}
