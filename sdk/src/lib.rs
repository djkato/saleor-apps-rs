pub mod apl;
#[cfg(feature = "bridge")]
pub mod bridge;
pub mod config;
pub mod headers;
pub mod locales;
pub mod manifest;
#[cfg(feature = "middleware")]
pub mod middleware;
pub mod webhooks;

use anyhow::bail;

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
pub struct SaleorApp {
    pub apl: Box<dyn APL>,
}

impl SaleorApp {
    pub fn new(config: &Config) -> anyhow::Result<SaleorApp> {
        use AplType::{File, Redis};
        fn decide_apl(config: &Config) -> anyhow::Result<Box<dyn APL>> {
            match config.apl {
                Redis => {
                    #[cfg(feature = "redis_apl")]
                    return Ok(Box::new(RedisApl::new(
                        &config.apl_url,
                        &config.app_api_base_url,
                    )?));

                    #[cfg(not(feature = "redis_apl"))]
                    {
                        bail!("Tried starting app with apl that wasn't present at compile time (cargo feature missing)")
                    }
                }
                File => {
                    #[cfg(feature = "file_apl")]
                    return Ok(Box::new(FileApl {
                        path: config.apl_url.to_owned(),
                    }));
                    #[cfg(not(feature = "file_apl"))]
                    {
                        bail!("Tried starting app with apl that wasn't present at compile time (cargo feature missing)")
                    }
                }
            }
        }
        let apl = decide_apl(config)?;
        Ok(SaleorApp { apl })
    }
}
