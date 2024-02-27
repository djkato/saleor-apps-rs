pub mod apl;
pub mod headers;
pub mod manifest;
pub mod webhooks;

use apl::APL;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone)]
pub struct SaleorApp<A: APL> {
    pub apl: A,
}
