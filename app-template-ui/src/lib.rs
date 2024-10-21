pub mod app;
pub mod error_template;
#[cfg(feature = "ssr")]
pub mod fileserv;
#[cfg(feature = "ssr")]
pub mod queries;

pub mod components;
pub mod routes;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount_to_body(App);
    use leptos::leptos_dom::logging::{console_error, console_log};
    console_log("starting main");
    use saleor_app_sdk::bridge::{
        event::{Event, PayloadRedirect},
        AppBridge,
    };
    match AppBridge::new(Some(true)) {
        Ok(mut app_bridge) => {
            console_log("App Bridge connected");
            _ = app_bridge.dispatch_event(Event::Redirect(PayloadRedirect {
                to: "/orders".to_owned(),
                new_context: None,
            }));
        }
        Err(e) => console_error(&format!("{:?}", e)),
    };
}
