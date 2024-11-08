use crate::{locales::LocaleCode, manifest::AppPermission};

use super::ThemeType;
// use bus::{Bus, BusReader};
use serde::{Deserialize, Serialize};
// use strum_macros::EnumIter;
// use web_sys::js_sys::Object;

// pub struct EventChannels {
//     pub handshake: Bus<PayloadHanshake>,
//     pub response: Bus<PayloadResponse>,
//     pub redirect: Bus<PayloadRedirect>,
//     pub theme: Bus<PayloadTheme>,
//     pub locale_changed: Bus<PayloadLocaleChanged>,
//     pub token_refreshed: Bus<PayloadTokenRefreshed>,
// }
//
// impl EventChannels {
//     pub fn new() -> Self {
//         Self {
//             handshake: Bus::new(10),
//             response: Bus::new(10),
//             redirect: Bus::new(10),
//             theme: Bus::new(10),
//             locale_changed: Bus::new(10),
//             token_refreshed: Bus::new(10),
//         }
//     }
//
//     pub fn subscribe_handshake(&mut self) -> BusReader<PayloadHanshake> {
//         self.handshake.add_rx()
//     }
//
//     pub fn subscribe_response(&mut self) -> BusReader<PayloadResponse> {
//         self.response.add_rx()
//     }
//
//     pub fn subscribe_redirect(&mut self) -> BusReader<PayloadRedirect> {
//         self.redirect.add_rx()
//     }
//
//     pub fn subscribe_theme(&mut self) -> BusReader<PayloadTheme> {
//         self.theme.add_rx()
//     }
//
//     pub fn subscribe_locale_changed(&mut self) -> BusReader<PayloadLocaleChanged> {
//         self.locale_changed.add_rx()
//     }
//
//     pub fn subscribe_token_refreshed(&mut self) -> BusReader<PayloadTokenRefreshed> {
//         self.token_refreshed.add_rx()
//     }
// }

// #[derive(EnumIter, Debug)]
// pub enum EventType {
//     Handshake,
//     Response,
//     Redirect,
//     Theme,
//     LocaleChanged,
//     TokenRefreshed,
// }

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
    pub version: f32,
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
