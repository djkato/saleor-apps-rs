use crate::routes::extensions::order_to_pdf::OrderToPdf;
use crate::routes::home::Home;
use leptos::leptos_dom::logging::{console_error, console_log, console_warn};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::{MetaTags, provide_meta_context};
use leptos_router::components::*;
use leptos_router::params::Params;
use leptos_router::*;
use saleor_app_sdk::bridge::action::{PayloadRedirect, PayloadRequestPermissions};
use saleor_app_sdk::bridge::event::Event;
use saleor_app_sdk::bridge::{AppBridge, dispatch_event, listen_to_events};
use serde::{Deserialize, Serialize};

#[derive(Params, PartialEq)]
pub struct UrlAppParams {
    slug: String,
}

#[component]
pub fn App() -> impl IntoView {
    let (bridge_read, bridge_set) = signal::<Option<AppBridge>>(None);
    Effect::new(move |_| match AppBridge::new(true) {
        Ok(bridge) => bridge_set(Some(bridge)),
        Err(e) => console_error(&format!("{:?}", e)),
    });

    Effect::new(move |_| {
        let manifest = use_context::<AppState>();
        if let Err(e) = dispatch_event(saleor_app_sdk::bridge::action::Action::RequestPermissions(
            PayloadRequestPermissions {
                permissions: manifest.map(|m| m.manifest.permissions).unwrap_or(vec![]),
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

    view! {
        // content for this welcome page
        <Router>
            <header class="h-12">
                <div class="h-full bg-default1 border-b-[1px] border-default1 px-4 py-2 flex justify-between items-center">
                    // <h2 class="">{context.map_or("[Cool App]".to_owned(), |c| c.manifest.name)}</h2>
                    <div>
                        <button on:click=move |_| {
                            dispatch_event(
                                    saleor_app_sdk::bridge::action::Action::Redirect(PayloadRedirect {
                                        to: format!(
                                            "/apps/{}/app",
                                            bridge_read
                                                .get()
                                                .map_or("undefined".to_owned(), |b| b.state.id),
                                        ),
                                        new_context: None,
                                    }),
                                )
                                .expect("failed sending redirect action");
                        }>Settings</button>

                    </div>
                    <div class="">

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
                    </div>
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

pub fn shell(options: LeptosOptions) -> impl IntoView {
    provide_meta_context();
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <link rel="stylesheet" id="leptos" href="/pkg/saleor-app-template-ui.css" />
                <link rel="shortcut icon" type="image/ico" href="/favicon.ico" />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[cfg(feature = "ssr")]
use surrealdb::Surreal;
#[cfg(feature = "ssr")]
use surrealdb::engine::any::Any;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "ssr", derive(axum::extract::FromRef))]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub config: saleor_app_sdk::config::Config,
    pub manifest: saleor_app_sdk::manifest::AppManifest,
    pub target_channel: String,
    #[cfg(feature = "ssr")]
    pub task_queue_sender: tokio::sync::mpsc::Sender<crate::server::task_handler::Event>,
    #[cfg(feature = "ssr")]
    pub saleor_app: std::sync::Arc<tokio::sync::Mutex<saleor_app_sdk::SaleorApp>>,
    #[cfg(feature = "ssr")]
    pub settings: AppSettings,
    #[cfg(feature = "ssr")]
    pub db_handle: Surreal<Any>,
}

#[cfg(feature = "ssr")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(rename = "heureka_target_folder")]
    pub target_folder: String,
    #[serde(rename = "heureka_allowed_host")]
    pub allowed_host: String,
    #[serde(rename = "heureka_variant_url_template")]
    pub variant_url_template: String,
    //eg. 23%
    #[serde(rename = "heureka_tax_rate")]
    pub tax_rate: String,
}

#[cfg(feature = "ssr")]
impl AppSettings {
    pub fn load() -> Result<Self, envy::Error> {
        _ = dotenvy::dotenv();
        envy::from_env::<AppSettings>()
    }
}
