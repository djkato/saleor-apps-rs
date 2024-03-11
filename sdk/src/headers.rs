use serde::{Deserialize, Serialize};

pub const SALEOR_DOMAIN_HEADER: &str = "saleor-domain";
pub const SALEOR_EVENT_HEADER: &str = "saleor-event";
pub const SALEOR_SIGNATURE_HEADER: &str = "saleor-signature";
pub const SALEOR_AUTHORIZATION_BEARER_HEADER: &str = "authorization-bearer";
pub const SALEOR_API_URL_HEADER: &str = "saleor-api-url";
pub const SALEOR_SCHEMA_VERSION: &str = "saleor-schema-version";

#[derive(Serialize, Deserialize)]
pub struct SaleorHeaders<'a> {
    #[serde(rename = "saleor-domain")]
    #[serde(alias = "x-saleor-domain")]
    domain: Option<&'a str>,
    #[serde(rename = "saleor-domain")]
    #[serde(alias = "x-saleor-domain")]
    authorization_bearer: Option<&'a str>,
    #[serde(rename = "saleor-domain")]
    #[serde(alias = "x-saleor-domain")]
    signature: Option<&'a str>,
    #[serde(rename = "saleor-domain")]
    #[serde(alias = "x-saleor-domain")]
    event: Option<&'a str>,
    saleor_api_url: Option<&'a str>,
    #[serde(rename = "content-length")]
    content_length: u16,
}
