use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use enum_iterator::{all, Sequence};
use iso_currency::Currency;
use saleor_app_sdk::{
    config::Config,
    manifest::{AppManifest, LocaleCode},
    SaleorApp,
};
use std::{str::FromStr, sync::Arc};
use tracing::{debug, level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;

use serde::{Deserialize, Serialize};
// Make our own error that wraps `anyhow::Error`.
pub struct AppError(pub anyhow::Error);

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

pub fn trace_to_std(config: &Config) -> anyhow::Result<()> {
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::DEBUG.into())
        .from_env()?
        .add_directive(format!("{}={}", env!("CARGO_PKG_NAME"), config.log_level).parse()?);
    tracing_subscriber::fmt()
        .with_max_level(config.log_level)
        .with_env_filter(filter)
        .with_target(true)
        .compact()
        .init();
    Ok(())
}

#[derive(Debug, Clone, Copy, Sequence, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PaymentMethodType {
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
    pub active_payment_methods: Vec<ActivePaymentMethod>,
    pub cod_extra_price_as_product_slug: Option<String>,
}

pub fn get_active_payment_methods_from_env() -> anyhow::Result<Vec<ActivePaymentMethod>> {
    _ = dotenvy::dotenv();
    //eg: "accreditation,cod,other,transfer"
    let env_methods = std::env::var("ACTIVE_PAYMENT_METHODS")?;
    let locale = std::env::var("LOCALE")?;
    let currencies = std::env::var("CURRENCIES")?;
    let locale = LocaleCode::from_str(&locale)?;
    let currencies = currencies.split(',').collect::<Vec<_>>();
    let currencies = currencies
        .iter()
        .map(|c| Currency::from_str(c))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| anyhow::anyhow!(format!("{:?}", e)))?;

    let str_types: Vec<_> = env_methods.split(',').collect();
    let payment_methods = str_types
        .iter()
        .flat_map(|s| all::<PaymentMethodType>().map(move |g| (s, g)))
        .filter_map(|(s, g)| match format!("{:?}", g).to_lowercase() == *s {
            true => Some(g),
            false => None,
        })
        .map(|g| ActivePaymentMethod {
            typ: g,
            name: match (g, &locale) {
                (PaymentMethodType::COD, LocaleCode::Sk) => "Dobierka".to_owned(),
                (PaymentMethodType::Cash, LocaleCode::Sk) => "Hotovosť".to_owned(),
                (PaymentMethodType::Transfer, LocaleCode::Sk) => "Bankový prevod".to_owned(),
                (PaymentMethodType::Inkaso, LocaleCode::Sk) => "Inkaso".to_owned(),
                (PaymentMethodType::Accreditation, LocaleCode::Sk) => "Vzajomný zápočet".to_owned(),
                (PaymentMethodType::Other, LocaleCode::Sk) => "Iné".to_owned(),
                (PaymentMethodType::COD, LocaleCode::En) => "Cash on delivery".to_owned(),
                (PaymentMethodType::Cash, LocaleCode::En) => "Cash".to_owned(),
                (PaymentMethodType::Transfer, LocaleCode::En) => "Bank transfer".to_owned(),
                (PaymentMethodType::Inkaso, LocaleCode::En) => "Encashment".to_owned(),
                (PaymentMethodType::Accreditation, LocaleCode::En) => "Mutual credit".to_owned(),
                (PaymentMethodType::Other, LocaleCode::En) => "Other".to_owned(),
                (g, l) => unimplemented!("Gateway {:?} in locale {:?} not implemented", g, l),
            },
        })
        .collect::<Vec<_>>();
    debug!(
        "active gateway types:{:?}\ncurrencies:{:?}\nlocale:{:?}",
        &payment_methods, &currencies, &locale
    );
    Ok(payment_methods)
}

#[derive(Debug, Clone, Serialize)]
pub struct ActivePaymentMethod {
    pub typ: PaymentMethodType,
    pub name: String,
}

#[derive(Serialize)]
pub struct PaymentGatewayInitializeSessionData {
    pub payment_methods: Vec<ActivePaymentMethod>,
}

#[derive(Deserialize, Serialize)]
pub struct TransactionInitializeSessionData {
    pub payment_method: PaymentMethodType,
}
