pub mod apl;
pub mod config;
pub mod headers;
pub mod manifest;
pub mod middleware;
pub mod webhooks;

use apl::{AplType, APL};
use config::Config;
use serde::{Deserialize, Serialize};

use crate::apl::{env_apl::EnvApl, file_apl::FileApl, redis_apl::RedisApl};

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
        use AplType::{Env, File, Redis};
        Ok(SaleorApp {
            apl: match config.apl {
                Redis => Box::new(RedisApl::new(&config.apl_url, &config.app_api_base_url)?),
                Env => Box::new(EnvApl {}),
                File => Box::new(FileApl {
                    path: "apl.txt".to_owned(),
                }),
            },
        })
    }
}
