mod app;
mod config;
mod routes;

use saleor_app_sdk::{
    apl::{env_apl::EnvApl, file_apl::FileApl, redis_apl::RedisApl, AplType, APL},
    manifest::{AppManifest, AppPermission, SaleorAppBranding, SaleorAppBrandingDefault},
    webhooks::{AsyncWebhookEventType, WebhookManifest},
    SaleorApp,
};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    app::{trace_to_std, AppState},
    config::Config,
    routes::create_routes,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::load()?;
    trace_to_std(&config);

    let apl: Box<dyn APL> = match config.apl {
        AplType::File => FileApl {
            path: "apl.json".to_owned(),
        },
        AplType::Redis => RedisApl::new(config.apl_url, config.app_api_base_url),
        AplType::Env => EnvApl {},
    };

    let saleor_app = SaleorApp { apl };

    let app_manifest = AppManifest {
        id: config.saleor_app_id.clone(),
        required_saleor_version: Some(config.required_saleor_version.clone()),
        name: env!("CARGO_PKG_NAME").to_owned(),
        about: Some(env!("CARGO_PKG_DESCRIPTION").to_owned()),
        author: Some(env!("CARGO_PKG_AUTHORS").to_owned()),
        version: env!("CARGO_PKG_VERSION").to_owned(),
        app_url: config.app_api_base_url.clone(),
        token_target_url: format!("{}/api/register", config.app_api_base_url.clone()),
        extensions: None,
        permissions: vec![AppPermission::ManageProducts],
        support_url: None,
        data_privacy: None,
        homepage_url: Some(env!("CARGO_PKG_HOMEPAGE").to_owned()),
        data_privacy_url: None,
        configuration_url: None,
        brand: Some(SaleorAppBranding {
            logo: SaleorAppBrandingDefault {
                default: format!("{}/logo.png", config.app_api_base_url),
            },
        }),
        webhooks: Some(vec![WebhookManifest {
            name: "GetProducts for demo rust app".to_owned(),
            query: r#"
                    subscription {
                        event {
                            ... on ProductUpdated {
                                product {
                                    id
                                    name
                                }
                            }
                        }
                    }
                    "#
            .to_owned(),
            is_active: Some(true),
            target_url: format!("{}/api/webhooks", config.app_api_base_url),
            sync_events: None,
            async_events: Some(vec![AsyncWebhookEventType::ProductCreated]),
        }]),
    };

    let app_state = AppState {
        manifest: app_manifest,
        config,
        saleor_app: Arc::new(Mutex::new(saleor_app)),
    };
    let app = create_routes(app_state);
    /* Router::new()
    .route("/api/manifest", get(manifest))
    .route("/api/register", post(register))
    .with_state(app_state);
    */

    //  let app = create_routes(app_state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    match axum::serve(listener, app).await {
        Ok(o) => Ok(o),
        Err(e) => anyhow::bail!(e),
    }
}
