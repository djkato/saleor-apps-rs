use super::{AppBridgeUser, ThemeType};
use crate::manifest::{AppPermission, LocaleCode};
use serde::{Deserialize, Serialize};

/**
 Events are what the dashboard sends and app receives on `window`
*/
#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type", content = "payload")]
#[serde(rename_all = "camelCase")]
pub enum Event {
    Handshake(PayloadHanshake),
    Response(PayloadResponse),
    Redirect(PayloadRedirect),
    Theme(PayloadTheme),
    LocaleChanged(PayloadLocaleChanged),
    TokenRefreshed(PayloadTokenRefreshed),
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct PayloadRequestPermissions {
    pub permissions: Vec<AppPermission>,
    pub redirect_path: String,
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct PayloadNotification {
    pub status: Option<NotificationStatus>,
    pub title: Option<String>,
    pub text: Option<String>,
    pub api_message: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum NotificationStatus {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PayloadHanshake {
    pub token: String,
    pub version: i32,
    pub saleor_version: Option<String>,
    pub dashboard_version: Option<String>,
}

#[derive(thiserror::Error, Debug)]
pub enum TokenIntoUserError {
    #[error("Failed fetching public key to validate JWK, {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("Failed parsing public key, {0}")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("Failed validating or parsing JWT, {0}")]
    CryptoError(#[from] jsonwebtoken::errors::Error),
    #[error("missing member in public key")]
    MissingKeyField,
}

impl PayloadHanshake {
    pub async fn token_into_user(&self) -> Result<AppBridgeUser, TokenIntoUserError> {
        use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
        use serde_json::Value;

        let jwks: Value = {
            let get_res = reqwest::get("http://localhost:8000/.well-known/jwks.json").await?;
            get_res.json::<Value>().await?
        };
        let nstr = jwks["keys"][0]["n"]
            .as_str()
            .ok_or(TokenIntoUserError::MissingKeyField)?;
        let estr = jwks["keys"][0]["e"]
            .as_str()
            .ok_or(TokenIntoUserError::MissingKeyField)?;

        let pubkey = DecodingKey::from_rsa_components(nstr, estr)?;
        let validation = Validation::new(Algorithm::RS256);
        let user = decode::<AppBridgeUser>(&self.token, &pubkey, &validation)?.claims;
        Ok(user)
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PayloadResponse {
    pub action_id: Option<String>,
    pub ok: bool,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PayloadRedirect {
    pub path: String,
    pub new_context: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct PayloadTheme {
    pub theme: ThemeType,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct PayloadLocaleChanged {
    pub locale: LocaleCode,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PayloadTokenRefreshed {
    pub token: String,
}
