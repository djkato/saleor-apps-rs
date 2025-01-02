use crate::error_template::{AppError, ErrorTemplate};
use crate::routes::extensions::order_to_pdf::OrderToPdf;
use crate::routes::home::Home;
use leptos::*;
use leptos_dom::logging::{console_error, console_log};
use leptos_meta::*;
use leptos_router::*;
use saleor_app_sdk::bridge::action::PayloadRequestPermissions;
use saleor_app_sdk::bridge::event::Event;
use saleor_app_sdk::bridge::{dispatch_event, listen_to_events, AppBridge};
use saleor_app_sdk::manifest::AppPermission;

#[derive(Params, PartialEq)]
pub struct UrlAppParams {
    slug: String,
}

#[component]
pub fn App() -> impl IntoView {
    let (bridge_read, bridge_set) = create_signal::<Option<AppBridge>>(None);

    create_effect(move |_| match AppBridge::new(true) {
        Ok(bridge) => bridge_set(Some(bridge)),
        Err(e) => console_error(&format!("{:?}", e)),
    });

    create_effect(move |_| {
        if let Err(e) = dispatch_event(saleor_app_sdk::bridge::action::Action::RequestPermissions(
            PayloadRequestPermissions {
                permissions: vec![AppPermission::ManageOrders, AppPermission::ManageProducts],
                redirect_path: None,
            },
        )) {
            console_error(&format!("{:?}", e));
        };
    });

    create_effect(move |_| {
        listen_to_events(move |event_res| match event_res {
            Ok(event) => match event {
                Event::Handshake(payload) => {
                    if let Some(mut bridge) = bridge_read.get_untracked() {
                        bridge.state.token = Some(payload.token);
                        bridge.state.dashboard_version = payload.dashboard_version;
                        bridge.state.saleor_version = payload.saleor_version;
                        bridge_set(Some(bridge))
                    }
                }
                Event::Response(_) => {
                    // console_log(&format!("front::App: {:?}", payload.ok));
                    if let Some(mut bridge) = bridge_read.get_untracked() {
                        bridge.state.ready = true;
                        bridge_set(Some(bridge))
                    }
                }
                Event::Redirect(payload) => {
                    console_log(&payload.to);
                }
                Event::Theme(payload) => {
                    if let Some(mut bridge) = bridge_read.get_untracked() {
                        bridge.state.theme = payload.theme;
                        bridge_set(Some(bridge))
                    }
                }
                Event::LocaleChanged(payload) => {
                    if let Some(mut bridge) = bridge_read.get_untracked() {
                        bridge.state.locale = payload.locale;
                        bridge_set(Some(bridge))
                    }
                }
                Event::TokenRefreshed(payload) => {
                    if let Some(mut bridge) = bridge_read.get_untracked() {
                        bridge.state.token = Some(payload.token);
                        bridge_set(Some(bridge))
                    }
                }
            },
            Err(e) => {
                console_error(&format!("front::App: {:?}", e));
            }
        })
    });

    provide_meta_context();
    view! {
        <Stylesheet id="leptos" href="/pkg/saleor-app-template-ui.css" />

        // sets the document title
        <Title text="Example UI App template in Rust" />

        // content for this welcome page
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! { <ErrorTemplate outside_errors /> }.into_view()
        }>
            <main class="p-4 md:p-8 md:px-16">
                <Routes>
                    <Route path="/" view=Home />
                    <Route path="/extensions/order_to_pdf" view=move || view!{<OrderToPdf bridge=bridge_read />}/>
                </Routes>
            </main>
        </Router>
    }
}

#[cfg(feature = "ssr")]
#[derive(Debug, Clone, axum::extract::FromRef)]
pub struct AppState {
    pub saleor_app: std::sync::Arc<tokio::sync::Mutex<saleor_app_sdk::SaleorApp>>,
    pub config: saleor_app_sdk::config::Config,
    pub manifest: saleor_app_sdk::manifest::AppManifest,
    pub leptos_options: LeptosOptions,
}
