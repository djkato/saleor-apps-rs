use serde::{Deserialize, Serialize};

use crate::manifest::AppPermission;

/**
 Actions are what the dashboard receives and app sends on `window.parent`
*/
#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type", content = "payload")]
#[serde(rename_all = "camelCase")]
pub enum Action {
    Redirect(PayloadRedirect),
    RequestPermissions(PayloadRequestPermissions),
    NotifyReady(String),
    Notification(PayloadNotification),
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct PayloadRequestPermissions {
    pub permissions: Vec<AppPermission>,
    // #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_path: Option<String>,
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

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PayloadRedirect {
    pub to: String,
    pub new_context: Option<bool>,
}
