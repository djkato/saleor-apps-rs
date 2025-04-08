#![allow(
    non_upper_case_globals,
    clippy::large_enum_variant,
    clippy::upper_case_acronyms,
    dead_code
)]
#![feature(let_chains)]

#[cfg(feature = "ssr")]
mod fallback;
#[cfg(feature = "ssr")]
mod queries;

#[cfg(feature = "ssr")]
mod server;

mod app;
mod components;
mod error_template;
mod routes;

#[tokio::main]
#[cfg(feature = "ssr")]
async fn main() -> Result<(), std::io::Error> {
    use app::*;
    use axum::{
        routing::{get, post},
        Router,
    };

    use fallback::file_and_error_handler;
    use leptos::{config::get_configuration, prelude::provide_context};
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use queries::event_products_updated::EVENTS_QUERY;
    use saleor_app_sdk::webhooks::{AsyncWebhookEventType, WebhookManifestBuilder};
    use saleor_app_sdk::{
        cargo_info,
        config::Config,
        manifest::{AppManifestBuilder, AppPermission},
        SaleorApp,
    };
    use server::task_handler::EventHandler;
    use tracing::error;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    use crate::routes::api::{manifest::manifest, register::register, webhooks::webhooks};

    //Leptos stuff
    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    // Saleor stuff
    let config = Config::load().unwrap();
    trace_to_std(&config).unwrap();
    let saleor_app = SaleorApp::new(&config).unwrap();

    let app_manifest = AppManifestBuilder::new(&config, cargo_info!())
        .add_permissions(vec![
            AppPermission::ManageProducts,
            AppPermission::ManageProductTypesAndAttributes,
        ])
        .add_webhook(
            WebhookManifestBuilder::new(&config)
                .set_query(EVENTS_QUERY)
                .add_async_events(vec![
                    AsyncWebhookEventType::ProductCreated,
                    AsyncWebhookEventType::ProductUpdated,
                    AsyncWebhookEventType::ProductDeleted,
                    AsyncWebhookEventType::CategoryCreated,
                    AsyncWebhookEventType::CategoryUpdated,
                    AsyncWebhookEventType::CategoryDeleted,
                    AsyncWebhookEventType::ProductVariantCreated,
                    AsyncWebhookEventType::ProductVariantUpdated,
                    AsyncWebhookEventType::ProductVariantDeleted,
                    AsyncWebhookEventType::ShippingZoneCreated,
                    AsyncWebhookEventType::ShippingZoneUpdated,
                    AsyncWebhookEventType::ShippingZoneDeleted,
                ])
                .build(),
        )
        .build()
        .expect("Failed building app manifest, contact app support plz");

    let (sender, receiver) = tokio::sync::mpsc::channel(100);


    let conn = surrealdb::Surreal::new::<surrealdb::engine::local::RocksDb>("./temp/db".to_owned())
        .await
        .expect("Failed creating DB connection");
    let app_state = AppState {
        db_handle: Arc::new(Mutex::new(conn)),
        task_queue_sender: sender,
        target_channel: match dotenvy::var("CHANNEL_SLUG") {
            Ok(v) => v,
            Err(_) => {
                error!("Missing channel slug, Saleor will soon deprecate product queries without channel specified.");
                "".to_string()
            }
        },

        manifest: app_manifest,
        config: config.clone(),
        saleor_app: Arc::new(Mutex::new(saleor_app)),
        leptos_options: leptos_options.clone(),
        settings: AppSettings::load().expect("Failed getting app settings from env"),
    };

    EventHandler::start(app_state.settings.clone(), receiver);

    let state_1 = app_state.clone();
    let app = Router::new()
        .leptos_routes_with_context(
            &app_state,
            routes,
            move || provide_context(state_1.clone()),
            move || shell(leptos_options.clone()),
        )
        .route(
            "/api/webhooks",
            post(webhooks), //.route_layer(middleware::from_fn(webhook_signature_verifier)),
        )
        .route(
            "/api/register",
            post(register), //.route_layer(middleware::from_fn(webhook_signature_verifier)),
        )
        .route("/api/manifest", get(manifest))
        .fallback(file_and_error_handler)
        .with_state(app_state.clone());
    // leptos_axum::file_and_error_handler(shell)

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
    // use leptos::leptos_dom::logging::{console_error, console_log};
    // console_log("starting main");
    // use saleor_app_sdk::bridge::AppBridge;
    // match AppBridge::new(Some(true)) {
    //     Ok(app_bridge) => {
    //         console_log("App Bridge connected");
    //     }
    //     Err(e) => console_error(e),
    // };
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
