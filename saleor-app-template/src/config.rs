use saleor_app_sdk::apl::AplType;
use serde::Deserialize;

use tracing::{debug, Level};

#[derive(Debug, Deserialize)]
#[serde(remote = "Level")]
pub enum LocalTracingLevel {
    TRACE,
    DEBUG,
    INFO,
    WARN,
    ERROR,
}

fn version_default() -> String {
    ">=3.11.7<4".to_owned()
}

#[derive(Deserialize, Debug, Clone)]
//#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct Config {
    #[serde(default = "version_default")]
    pub required_saleor_version: String,
    pub saleor_app_id: String,
    pub app_api_base_url: String,
    pub apl: AplType,
    pub apl_url: String,
    #[serde(with = "LocalTracingLevel")]
    pub log_level: tracing::Level,
}

impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Config {
    pub fn load() -> Result<Self, envy::Error> {
        dotenvy::dotenv().unwrap();
        let env = envy::from_env::<Config>();
        if let Ok(e) = &env {
            debug!("{}", e);
        }
        env
    }
}
