pub mod action;
pub mod event;

use std::str::FromStr;

use action::Action;
use serde::{Deserialize, Serialize};
use strum_macros::{EnumString, IntoStaticStr};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};

use crate::manifest::{AppPermission, LocaleCode};

use self::event::Event;
use web_sys::{console, js_sys::JSON};

#[derive(Default, Debug, Clone)]
pub struct AppBridge {
    pub state: AppBridgeState,
    pub referer_origin: Option<String>,
    // pub event_channels: EventChannels,
    /**
     * Should automatically emit Actions.NotifyReady.
     * If app loading time is longer, this can be disabled and sent manually.
     */
    auto_notify_ready: bool,
}

#[derive(Default, Debug, Clone)]
pub struct AppBridgeState {
    pub token: Option<String>,
    pub id: String,
    pub ready: bool,
    pub domain: String,
    pub path: String,
    pub theme: ThemeType,
    pub locale: LocaleCode,
    pub saleor_api_url: String,
    /**pub
     * Versions of Saleor that app is mounted. Passed from the Dashboard.pub
     * Works form Saleor 3.15pub
     */
    pub saleor_version: Option<String>,
    pub dashboard_version: Option<String>,
    pub user: Option<AppBridgeUser>,
    pub app_permissions: Option<Vec<AppPermission>>,
}

impl AppBridgeState {
    pub fn from_window() -> Result<Self, JsValue> {
        let mut state = AppBridgeState::default();
        let window = web_sys::window().ok_or(JsValue::from_str("Missing window"))?;
        let href = window.location().href()?;
        let url = web_sys::Url::new(&href)?;

        let saleor_api_url = url
            .search_params()
            .get(AppIframeParams::SaleorApiUrl.into());
        let id = url.search_params().get(AppIframeParams::Id.into());
        let theme = url.search_params().get(AppIframeParams::Theme.into());
        let domain = url.search_params().get(AppIframeParams::Domain.into());
        let locale = url.search_params().get(AppIframeParams::Locale.into());

        if let Some(id) = id {
            state.id = id
        }
        if let Some(saleor_api_url) = saleor_api_url {
            state.saleor_api_url = saleor_api_url
        }
        if let Some(theme) = theme {
            if let Ok(theme_type) = ThemeType::from_str(&theme) {
                state.theme = theme_type
            }
        }
        if let Some(domain) = domain {
            state.domain = domain
        }
        if let Some(locale) = locale {
            if let Ok(loc) = LocaleCode::from_str(&locale) {
                state.locale = loc
            }
        }
        // debug!("state from window: {:?}", &state);
        console::log_1(&format!("state from window: {:?}", &state).into());
        Ok(state)
    }
}

#[derive(Default, Debug, Clone)]
pub struct AppBridgeUser {
    /**
     * Original permissions of the user that is using the app.
     * *Not* the same permissions as the app itself.
     *
     * Can be used by app to check if user is authorized to perform
     * domain specific actions
     */
    pub permissions: Vec<AppPermission>,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, EnumString, IntoStaticStr)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum AppIframeParams {
    Id,
    Theme,
    Domain,
    SaleorApiUrl,
    Locale,
}

#[derive(Debug, Serialize, Deserialize, EnumString, Clone, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThemeType {
    #[default]
    Light,
    Dark,
}

impl AppBridge {
    pub fn new(auto_notify_ready: bool) -> Result<Self, AppBridgeError> {
        // debug!("creating app bridge");
        console::log_1(&"creating app bridge".into());
        if web_sys::Window::is_type_of(&JsValue::from_str("undefined")) {
            // error!("Window is undefined");
            console::log_1(&"Window is undefined".into());
            return Err(AppBridgeError::WindowIsUndefined);
        }
        let referrer = web_sys::window().and_then(|w| {
            w.document()
                .and_then(|d| web_sys::Url::new(&d.referrer()).ok().map(|u| u.origin()))
        });
        if referrer.is_none() {
            // warn!("Referrer origin is none");
            console::log_1(&"Referrer origin is none".into());
        }

        let bridge = Self {
            auto_notify_ready,
            state: match AppBridgeState::from_window() {
                Ok(s) => s,
                Err(e) => return Err(AppBridgeError::JsValue(e)),
            },
            referer_origin: referrer,
        };
        if bridge.auto_notify_ready {
            dispatch_event(Action::NotifyReady("".into()))?;
        }
        Ok(bridge)
    }
}

/**
 * make sure to keep the returned closure handle safe, once it deallocs events will no longer
 * trigger
 */
pub fn listen_to_events(
    mut on_event: impl FnMut(Result<Event, serde_wasm_bindgen::Error>) + 'static,
) -> Result<Closure<dyn FnMut(JsValue)>, AppBridgeError> {
    let window = web_sys::window().ok_or(AppBridgeError::WindowIsUndefined)?;
    let cb = Closure::wrap(Box::new(move |e: JsValue| {
        web_sys::console::log_1(
            &format!(
                "sdk::bridge::listen_to_events: {:?}",
                &JSON::stringify(&web_sys::js_sys::Reflect::get(&e, &"data".into()).unwrap())
                    .unwrap()
            )
            .into(),
        );
        let event_data: Result<Event, _> = serde_wasm_bindgen::from_value(
                web_sys::js_sys::Reflect::get(&e, &"data".into())
                    .expect("Closure should've received object with .data property, but didn't, saleor plz fix?"),
            );
        // web_sys::console::log_1(&format!("{:?}", &event_data).into());
        on_event(event_data);
    }) as Box<dyn FnMut(JsValue)>);

    window
        .add_event_listener_with_callback("message", cb.as_ref().unchecked_ref())
        .map_err(AppBridgeError::JsValue)?;
    Ok(cb)
}

pub fn dispatch_event(action: Action) -> Result<(), AppBridgeError> {
    let window = web_sys::window().ok_or(AppBridgeError::WindowIsUndefined)?;
    let parent = match window.parent() {
        Ok(p) => p.ok_or(AppBridgeError::WindowParentIsUndefined)?,
        Err(e) => return Err(AppBridgeError::JsValue(e)),
    };
    // let message = JsValue::from(&event);
    let message = serde_wasm_bindgen::to_value(&action)?;
    web_sys::console::log_1(&message);
    parent
        .post_message(&message, "*")
        .map_err(AppBridgeError::JsValue)?;
    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum AppBridgeError {
    #[error("failed serializing event from window")]
    SerdeError(#[from] serde_wasm_bindgen::Error),
    #[error("Something went wrong with serde_json::to_string(&event)")]
    FailedPayloadToJsonStringification(#[from] serde_json::Error),
    #[error("Windows parent is missing, meaning the app is probably not embedded in Iframe")]
    WindowParentIsUndefined,
    #[error("Window is typeof undefined. Probably means AppBridge::new() is being called outside of a browser")]
    WindowIsUndefined,
    #[error("JS error")]
    JsValue(JsValue),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SaleorIframeEvent {
    pub origin: String,
    pub data: Event,
}
