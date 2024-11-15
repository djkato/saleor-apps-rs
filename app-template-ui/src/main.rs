#![allow(
    non_upper_case_globals,
    clippy::large_enum_variant,
    clippy::upper_case_acronyms,
    dead_code
)]
#![feature(let_chains)]

#[cfg(feature = "ssr")]
mod fileserv;
#[cfg(feature = "ssr")]
mod queries;

mod app;
mod components;
mod error_template;
mod routes;

#[tokio::main]
#[cfg(feature = "ssr")]
async fn main() -> Result<(), std::io::Error> {
    use app::*;
    use axum::{
        middleware,
        routing::{get, post},
        Router,
    };
    use fileserv::file_and_error_handler;
    use leptos::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use saleor_app_sdk::{
        cargo_info,
        config::Config,
        manifest::{AppManifestBuilder, AppPermission},
        webhooks::{AsyncWebhookEventType, WebhookManifestBuilder},
        SaleorApp,
    };
    use saleor_app_sdk::{
        manifest::{extension::AppExtensionBuilder, AppExtensionMount, AppExtensionTarget},
        middleware::verify_webhook_signature::webhook_signature_verifier,
    };
    use std::sync::Arc;
    use tokio::sync::Mutex;

    use crate::routes::api::{manifest::manifest, register::register, webhooks::webhooks};

    //Leptos stuff
    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(App);

    // Saleor stuff
    let config = Config::load().unwrap();
    trace_to_std(&config).unwrap();
    let saleor_app = SaleorApp::new(&config).unwrap();

    let app_manifest = AppManifestBuilder::new(&config, cargo_info!())
        .add_permissions(vec![
            AppPermission::ManageProducts,
            AppPermission::ManageOrders,
            AppPermission::ManageProductTypesAndAttributes,
        ])
        .add_extension(
            AppExtensionBuilder::new()
                .set_url("/extensions/order_to_pdf")
                .set_label("Order to PDF")
                .add_permissions(vec![
                    AppPermission::ManageOrders,
                    AppPermission::ManageProducts,
                    AppPermission::ManageProductTypesAndAttributes,
                ])
                .set_mount(AppExtensionMount::OrderDetailsMoreActions)
                .set_target(AppExtensionTarget::Popup)
                .build(),
        )
        .build()
        .expect("Manifest has invalid parameters");

    let app_state = AppState {
        manifest: app_manifest,
        config: config.clone(),
        saleor_app: Arc::new(Mutex::new(saleor_app)),
        leptos_options,
    };

    let state_1 = app_state.clone();
    let app = Router::new()
        .leptos_routes_with_context(
            &app_state,
            routes,
            move || provide_context(state_1.clone()),
            App,
        )
        .fallback(file_and_error_handler)
        .route(
            "/api/webhooks",
            post(webhooks)//.route_layer(middleware::from_fn(webhook_signature_verifier)),
        )
        .route(
            "/api/register",
            post(register)//.route_layer(middleware::from_fn(webhook_signature_verifier)),

        )
        .route("/api/manifest", get(manifest))
        .with_state(app_state.clone());

    let listener = tokio::net::TcpListener::bind(
        "0.0.0.0:".to_owned()
            + config
                .app_api_base_url
                .split(':')
                .collect::<Vec<_>>()
                .get(2)
                .unwrap_or(&"3000"),
    )
    .await?;
    tracing::debug!("listening on {}", listener.local_addr()?);

    let _ = axum::serve(listener, app.into_make_service()).await;
    Ok(())
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    use leptos::leptos_dom::logging::{console_error, console_log};
    console_log("starting main");
    use saleor_app_sdk::bridge::AppBridge;
    match AppBridge::new(Some(true)) {
        Ok(app_bridge ) => {
            console_log("App Bridge connected");
        }
        Err(e) => console_error(e)
    };
    // no client-side main function
    // unless we want this to work with e.g., Trunk for a purely client-side app
    // see lib.rs for hydration function instead
}

#[cfg(feature = "ssr")]
use saleor_app_sdk::config::Config;
#[cfg(feature = "ssr")]
pub fn trace_to_std(config: &Config) -> Result<(), envy::Error> {
    use tracing::level_filters::LevelFilter;
    use tracing_subscriber::EnvFilter;

    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::DEBUG.into())
        .from_env()
        .unwrap()
        .add_directive(
            format!("{}={}", env!("CARGO_PKG_NAME"), config.log_level)
                .parse()
                .unwrap(),
        );
    tracing_subscriber::fmt()
        .with_max_level(config.log_level)
        .with_env_filter(filter)
        .with_target(true)
        .compact()
        .init();
    Ok(())
}
