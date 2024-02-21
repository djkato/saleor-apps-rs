mod app;
mod config;
mod routes;
mod saleor;

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    app::{trace_to_std, AppState},
    config::Config,
    routes::create_routes,
    saleor::{
        AppManifest, AppPermission, AsyncWebhookEventType, RedisApl, SaleorApp, WebhookManifest,
    },
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::load()?;
    trace_to_std(&config);
    let saleor_app = SaleorApp::<RedisApl> {
        apl: RedisApl::new(config.apl_url.clone(), config.app_api_base_url.clone())?,
    };
    let app_manifest = AppManifest {
        id: config.saleor_app_id.clone(),
        required_saleor_version: Some(config.required_saleor_version.clone()),
        name: config.app_name.clone(),
        about: config.app_description.clone(),
        brand: None,
        author: config.app_author.clone(),
        version: env!("CARGO_PKG_VERSION").to_owned(),
        app_url: config.app_api_base_url.clone(),
        token_target_url: format!("{}/api/register", config.app_api_base_url.clone()),
        extensions: None,
        permissions: vec![AppPermission::ManageProducts],
        support_url: None,
        data_privacy: None,
        homepage_url: None,
        data_privacy_url: None,
        configuration_url: None,
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
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app);
    Ok(())
}
