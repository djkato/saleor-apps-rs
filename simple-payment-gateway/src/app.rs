use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use enum_iterator::{all, Sequence};
use std::{fmt::Display, str::FromStr, sync::Arc};

use saleor_app_sdk::{config::Config, locales::LocaleCode, manifest::AppManifest, SaleorApp};
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize,
};
// Make our own error that wraps `anyhow::Error`.
pub struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

pub fn trace_to_std(config: &Config) {
    tracing_subscriber::fmt()
        .with_max_level(config.log_level)
        .with_target(false)
        .init();
}

#[derive(Debug, Clone, Sequence, Serialize)]
pub enum GatewayType {
    Accreditation,
    Cash,
    /**
    Acronym for Cash on Delivery
    */
    COD,
    Inkaso,
    Other,
    Transfer,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub saleor_app: Arc<tokio::sync::Mutex<SaleorApp>>,
    pub config: Config,
    pub manifest: AppManifest,
    pub active_gateways: Vec<ActiveGateway>,
}

pub fn get_active_gateways_from_env() -> anyhow::Result<Vec<ActiveGateway>> {
    dotenvy::dotenv()?;
    //eg: "accreditation,cod,other,transfer"
    let env_types = std::env::var("ACTIVE_GATEWAYS")?;
    let locale = std::env::var("LOCALE")?;
    let locale = match locale.as_str() {
        "SK" => LocaleCode::Sk,
        "EN" => LocaleCode::En,
        l => unimplemented!("Locale {l} not implemented"),
    };

    let str_types: Vec<_> = env_types.split(",").collect();
    let gateway_types = str_types
        .iter()
        .zip(all::<GatewayType>())
        .filter_map(|(s, g)| match format!("{:?}", g).to_lowercase() == *s {
            true => Some(g),
            false => None,
        })
        .map(|g| )
        .collect::<Vec<_>>();

    todo!()
}

#[derive(Debug, Clone, Serialize)]
pub struct ActiveGateway {
    pub gateway_type: GatewayType,
    pub id: String,
    pub name: String,
    pub currencies: Vec<String>,
    //don't need this one yet
    pub config: [(); 0],
}
impl ActiveGateway{
    fn from_gateway_type(ty: &GatewayType) -> Self {
        all_currencies = 
        match type {
            
        }
    }
}
