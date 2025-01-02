use super::ThemeType;
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

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct PayloadHanshake {
    pub token: String,
    pub version: i32,
    pub saleor_version: Option<String>,
    pub dashboard_version: Option<String>,
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
    pub to: String,
    pub new_context: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct PayloadTheme {
    pub theme: ThemeType,
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct PayloadLocaleChanged {
    pub locale: LocaleCode,
}
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PayloadTokenRefreshed {
    pub token: String,
}
