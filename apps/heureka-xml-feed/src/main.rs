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
        Router,
        routing::{get, post},
    };

    use fallback::file_and_error_handler;
    use leptos::{config::get_configuration, prelude::provide_context};
    use leptos_axum::{LeptosRoutes, generate_route_list};
    use queries::event_products_updated::EVENTS_QUERY;
    use saleor_app_sdk::webhooks::{AsyncWebhookEventType, WebhookManifestBuilder};
    use saleor_app_sdk::{
        SaleorApp, cargo_info,
        config::Config,
        manifest::{AppManifestBuilder, AppPermission},
    };
    use server::task_handler::EventHandler;
    use std::sync::Arc;
    use surrealdb::engine::any;
    use tokio::sync::Mutex;
    use tower_http::trace::{
        DefaultMakeSpan, DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer,
    };
    use tracing::{Level, error};

    use crate::routes::api::{manifest::manifest, register::register, webhooks::webhooks};

    //Leptos stuff
    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    // let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    // Saleor stuff
    let config = Config::load().unwrap();
    trace_to_std(&config).unwrap();
    let saleor_app = SaleorApp::new(&config).unwrap();

    let app_manifest = AppManifestBuilder::new(&config, cargo_info!())
        .add_permissions(vec![
            AppPermission::ManageProducts,
            AppPermission::ManageProductTypesAndAttributes,
            AppPermission::ManageShipping,
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

    let db_handle = any::connect(dotenvy::var("SURREALDB_URL").unwrap_or("memory".to_owned()))
        .await
        .expect("Failed creating DB connection");

    db_handle
        .use_ns("saleor")
        .use_db("saleor-app-heureka-xml-feed")
        .await
        .expect("Failed switching DB NS & DB");

    db_handle
        .query(include_str!("../saleor-heureka-testing-2025-05-10.surql"))
        .await
        .expect("Failed upserting init tables for DB");

    let app_state = AppState {
        db_handle: db_handle.clone(),
        task_queue_sender: sender,
        target_channel: match dotenvy::var("CHANNEL_SLUG") {
            Ok(v) => v,
            Err(_) => {
                error!(
                    "Missing channel slug, Saleor will soon deprecate product queries without channel specified."
                );
                "".to_string()
            }
        },

        manifest: app_manifest,
        config: config.clone(),
        saleor_app: Arc::new(Mutex::new(saleor_app)),
        leptos_options: leptos_options.clone(),
        settings: AppSettings::load().expect("Failed getting app settings from env"),
    };

    EventHandler::start(app_state.settings.clone(), receiver, db_handle);

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
        .with_state(app_state.clone())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new())
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO))
                .on_failure(DefaultOnFailure::new().level(Level::ERROR)),
        );

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

#[cfg(feature = "ssr")]
use saleor_app_sdk::config::Config;
#[cfg(feature = "ssr")]
pub fn trace_to_std(config: &Config) -> Result<(), envy::Error> {
    use tracing_subscriber::{
        EnvFilter, fmt, layer::SubscriberExt, registry, util::SubscriberInitExt,
    };

    registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            format!(
                "{}={},tower_http=debug,axum::rejection=trace",
                env!("CARGO_CRATE_NAME"),
                config.log_level.to_string()
            )
            .into()
        }))
        .with(fmt::layer())
        .init();

    Ok(())
}
