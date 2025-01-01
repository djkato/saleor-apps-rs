#![allow(
    non_upper_case_globals,
    clippy::large_enum_variant,
    clippy::upper_case_acronyms,
    dead_code
)]
#![feature(let_chains)]
// #![deny(clippy::unwrap_used, clippy::expect_used)]
mod app;
mod queries;
mod routes;
mod updater;

use app::ManipulatorConfig;
use axum::Router;
use saleor_app_sdk::{
    config::Config,
    manifest::{cargo_info, AppManifestBuilder, AppPermission},
    SaleorApp,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};

use crate::{
    app::{trace_to_std, AppState},
    routes::create_routes,
};

#[tokio::main]
async fn main() {
    debug!("Creating configs...");
    let config = Config::load().unwrap();
    trace_to_std(&config).unwrap();
    let manipulator_config = ManipulatorConfig::load().unwrap();

    let app = create_app(&config, manipulator_config).await;

    let listener = tokio::net::TcpListener::bind(
        "0.0.0.0:".to_owned()
            + config
                .app_api_base_url
                .split(':')
                .collect::<Vec<_>>()
                .get(2)
                .unwrap_or(&"3000"),
    )
    .await
    .unwrap();
    info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn create_app(config: &Config, manipulator_config: ManipulatorConfig) -> Router {
    let saleor_app = SaleorApp::new(config).unwrap();

    debug!("Creating saleor App...");
    let app_manifest = AppManifestBuilder::new(config, cargo_info!())
        .add_permissions(vec![
            AppPermission::ManageProducts,
            AppPermission::HandleTaxes,
            AppPermission::ManageTaxes,
            AppPermission::ManageChannels,
            AppPermission::ManageProductTypesAndAttributes,
        ])
        .build();
    debug!("Created AppManifest...");

    let app_state = AppState {
        manipulator: manipulator_config,
        manifest: app_manifest,
        config: config.clone(),
        target_channel: match dotenvy::var("CHANNEL_SLUG") {
            Ok(v) => v,
            Err(_) => {
                panic!("Missing channel slug. Slug is needed for price obtainment.");
            }
        },
        saleor_app: Arc::new(Mutex::new(saleor_app)),
    };
    debug!("Created AppState...");
    create_routes(app_state)
}
