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
use saleor_app_sdk::manifest::{AppPermission, LocaleCode};
use serde::{Deserialize, Serialize};
use strum_macros::EnumString;

#[derive(Params, PartialEq)]
pub struct UrlAppParams {
    slug: String,
}

#[component]
pub fn App() -> impl IntoView {
    let (bridge_read, bridge_set) = create_signal::<Option<AppBridge>>(None);
    let context = use_context::<AppState>();
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
        <Stylesheet id="leptos" href="/pkg/saleor-app-template-ui.css" />

        // sets the document title
        <Title text="Example UI App template in Rust" />

        // content for this welcome page
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! { <ErrorTemplate outside_errors /> }.into_view()
        }>
            <header class="h-12">
                <div class="h-full bg-default1 border-b-[1px] border-default1 px-4 py-2 flex justify-between items-center">
                    <h2 class="">
                        { context.map_or("[Cool App]".to_owned(), |c| c.manifest.name)}
                    </h2>
                    <span class="">
                        {move || match bridge_read.get() {
                            Some(bridge) => bridge.state.user.map_or("[Loading bridge...]".into(), |u|"Welcome, ".to_owned()+ &u.email),
                            None => "[Not authenticated]".into()
                        }}
                    </span>
                </div>
            </header>
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
use saleor_app_sdk::settings_manager::metadata::MetadataSettingsManager;

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
