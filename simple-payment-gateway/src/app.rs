use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use enum_iterator::{all, Sequence};
use iso_currency::Currency;
use std::{str::FromStr, sync::Arc};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

use saleor_app_sdk::{
    config::Config, locales::LocaleCode, manifest::AppManifest,
    webhooks::sync_response::PaymentGateway, SaleorApp,
};
use serde::Serialize;
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
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::DEBUG.into())
        .from_env()
        .unwrap()
        .add_directive(
            format!("{}={}", env!("CARGO_PKG_NAME"), config.log_level)
                .parse()
                .unwrap(),
        );
    tracing_subscriber::fmt()
        .with_max_level(config.log_level)
        .with_env_filter(filter)
        .with_target(true)
        .compact()
        .init();
}

#[derive(Debug, Clone, Sequence, Serialize)]
#[serde(rename_all = "lowercase")]
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
    _ = dotenvy::dotenv();
    //eg: "accreditation,cod,other,transfer"
    let env_types = std::env::var("ACTIVE_GATEWAYS")?;
    let locale = std::env::var("LOCALE")?;
    let locale = LocaleCode::from_str(&locale)?;
    let currencies = std::env::var("CURRENCIES")?;
    let currencies = currencies.split(',').collect::<Vec<_>>();
    let currencies = currencies
        .iter()
        .map(|c| Currency::from_str(c))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| anyhow::anyhow!(format!("{:?}", e)))?;

    let str_types: Vec<_> = env_types.split(',').collect();
    let gateway_types = str_types
        .iter()
        .zip(all::<GatewayType>())
        .filter_map(|(s, g)| match format!("{:?}", g).to_lowercase() == *s {
            true => Some(g),
            false => None,
        })
        .map(|g| ActiveGateway {
            gateway_type: g.clone(),
            gateway: PaymentGateway {
                currencies: currencies.clone(),
                id: format!("{:?}", &g).to_lowercase(),
                config: vec![],
                name: match (g, &locale) {
                    (GatewayType::COD, LocaleCode::Sk) => "Dobierka".to_owned(),
                    (GatewayType::Cash, LocaleCode::Sk) => "Hotovosť".to_owned(),
                    (GatewayType::Transfer, LocaleCode::Sk) => "Bankový prevod".to_owned(),
                    (GatewayType::Inkaso, LocaleCode::Sk) => "Inkaso".to_owned(),
                    (GatewayType::Accreditation, LocaleCode::Sk) => "Vzajomný zápočet".to_owned(),
                    (GatewayType::Other, LocaleCode::Sk) => "Iné".to_owned(),
                    (GatewayType::COD, LocaleCode::En) => "Cash on delivery".to_owned(),
                    (GatewayType::Cash, LocaleCode::En) => "Cash".to_owned(),
                    (GatewayType::Transfer, LocaleCode::En) => "Bank transfer".to_owned(),
                    (GatewayType::Inkaso, LocaleCode::En) => "Encashment".to_owned(),
                    (GatewayType::Accreditation, LocaleCode::En) => "Mutual credit".to_owned(),
                    (GatewayType::Other, LocaleCode::En) => "Other".to_owned(),
                    (g, l) => unimplemented!("Gateway {:?} in locale {:?} not implemented", g, l),
                },
            },
        })
        .collect::<Vec<_>>();

    Ok(gateway_types)
}

#[derive(Debug, Clone, Serialize)]
pub struct ActiveGateway {
    pub gateway_type: GatewayType,
    pub gateway: PaymentGateway,
}
