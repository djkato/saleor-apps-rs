use serde::Deserialize;

#[cfg(feature = "tracing")]
use tracing::{debug, Level};

use crate::apl::AplType;

#[cfg(feature = "tracing")]
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
    pub app_api_base_url: String,
    pub app_iframe_base_url: String,
    pub apl: AplType,
    pub apl_url: String,
    #[cfg(feature = "tracing")]
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
        _ = dotenvy::dotenv();
        let env = envy::from_env::<Config>();
        #[cfg(feature = "tracing")]
        debug!("{:?}", &env);
        env
    }
}
