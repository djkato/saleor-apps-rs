use crate::routes::extensions::order_to_pdf::OrderToPdf;
use crate::routes::home::Home;
use leptos::leptos_dom::logging::{console_error, console_log, console_warn};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::{provide_meta_context, MetaTags};
use saleor_app_sdk::bridge::action::PayloadRequestPermissions;
use saleor_app_sdk::bridge::event::Event;
use saleor_app_sdk::bridge::{dispatch_event, listen_to_events, AppBridge};
use saleor_app_sdk::manifest::LocaleCode;
use serde::{Deserialize, Serialize};
use strum_macros::EnumString;
use leptos_router:: params::Params;
use leptos_router::components::*;
use leptos_router::*;

#[derive(Params, PartialEq)]
pub struct UrlAppParams {
    slug: String,
}

#[component]
pub fn App() -> impl IntoView {
    let (bridge_read, bridge_set) =  signal::<Option<AppBridge>>(None);
    let context = use_context::<AppState>();
    Effect::new(move |_| match AppBridge::new(true) {
        Ok(bridge) => bridge_set(Some(bridge)),
        Err(e) => console_error(&format!("{:?}", e)),
    });

    Effect::new(move |_| {
    let manifest = use_context::<AppState>();
        if let Err(e) = dispatch_event(saleor_app_sdk::bridge::action::Action::RequestPermissions(
            PayloadRequestPermissions {
                permissions: manifest.map(|m|m.manifest.permissions).unwrap_or(vec![]),
                redirect_path: None,
            },
        )) {
            console_error(&format!("{:?}", e));
        };
    });

    Effect::new(move |_| {
        listen_to_events(move |event_res| match event_res {
            Ok(event) => {
                match event {
                    Event::Handshake(payload) => {
                        if let Some(mut bridge) = bridge_read.get_untracked() {
                            let payload2 = payload.clone();
                            spawn_local(async move {
                                match payload2.token_into_user().await {
                                    Ok(user) => {
                                        console_log(&format!(
                                            "setting bridge user to:\n {:?}",
                                            &user
                                        ));
                                        bridge.state.user = Some(user);
                                    }
                                    Err(e) => {
                                        console_error(&format!(
                                            "failed converting JWT into user data, {e:?}"
                                        ));
                                        bridge.state.user = None;
                                    }
                                };
                                bridge.state.token = Some(payload.clone().token);
                                bridge.state.dashboard_version = payload.dashboard_version;
                                bridge.state.saleor_version = payload.saleor_version;
                                //if for some reason it unsets sometimes
                                bridge.state.ready = true;
                                bridge_set(Some(bridge));
                            });
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
                        console_log(&payload.path);
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
                }
            }
            Err(e) => {
                console_warn(&format!("front::App: {:?}", e));
            }
        })
    });

    provide_meta_context();
    view! {
        // content for this welcome page
        <Router>
            <header class="h-12">
                <div class="h-full bg-default1 border-b-[1px] border-default1 px-4 py-2 flex justify-between items-center">
                    <h2 class="">{context.map_or("[Cool App]".to_owned(), |c| c.manifest.name)}</h2>
                    <span class="">
                        {move || match bridge_read.get() {
                            Some(bridge) => {
                                bridge
                                    .state
                                    .user
                                    .map_or(
                                        "[Loading bridge...]".into(),
                                        |u| "Welcome, ".to_owned() + &u.email,
                                    )
                            }
                            None => "[Not authenticated]".into(),
                        }}
                    </span>
                </div>
            </header>
            <main class="p-4 md:p-8 md:px-16">
                <Routes fallback=|| view! { <p>"Page not found"</p> }>
                    <Route path=path!("/") view=Home />
                    <Route
                        path=path!("/extensions/order_to_pdf")
                        view=move || view! { <OrderToPdf bridge=bridge_read /> }
                    />
                </Routes>
            </main>
        </Router>
    }
}
#[cfg(feature = "ssr")]
use saleor_app_sdk::settings_manager::metadata::MetadataSettingsManager;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options islands=true />
                <link rel="stylesheet" id="leptos" href="/pkg/portfolio.css" />
                <link rel="shortcut icon" type="image/ico" href="/favicon.ico" />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "ssr", derive(axum::extract::FromRef))]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub config: saleor_app_sdk::config::Config,
    pub manifest: saleor_app_sdk::manifest::AppManifest,
    #[cfg(feature = "ssr")]
    pub saleor_app: std::sync::Arc<tokio::sync::Mutex<saleor_app_sdk::SaleorApp>>,
    #[cfg(feature = "ssr")]
    pub settings: std::sync::Arc<
        tokio::sync::Mutex<Option<MetadataSettingsManager<AppSettingsKey, AppSettings>>>,
    >,
}

#[derive(Hash, PartialEq, Eq, Debug, Clone, EnumString, strum_macros::Display)]
pub enum AppSettingsKey {
    Global,
    //ID
    User(String),
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub enum AppSettings {
    Global(GlobalAppSettings),
    UserSettings(UserAppSettings),
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct GlobalAppSettings {
    idk: bool,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct UserAppSettings {
    pub locale: Option<LocaleCode>,
    pub active_pdf_fields: Vec<OrderDetailField>,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub enum OrderDetailField {
    VariantName,
    ProductName,
    VariantSKU,
    VariantOrderAmount,
    VariantWarehouseAmount,
    VariantPriceSingleGross,
    LinePriceGross,
    ObtainmentMethod,
    PaymentMethod,
}
