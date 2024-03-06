mod app;
mod queries;
mod routes;

use saleor_app_sdk::{
    config::Config,
    manifest::{AppManifest, AppPermission},
    webhooks::{AsyncWebhookEventType, WebhookManifest},
    SaleorApp,
};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    app::{trace_to_std, AppState},
    routes::create_routes,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::load()?;
    trace_to_std(&config);

    let saleor_app = SaleorApp::new(&config)?;

    let app_manifest = AppManifest::new(&config)
        .add_webhook(
            WebhookManifest::new(&config)
                .set_query(
                    r#"
                    subscription QueryProductsChanged {
                      event {
                        ... on ProductUpdated {
                          product {
                            ... BaseProduct
                          }
                        }
                        ... on ProductCreated {
                          product {
                            ... BaseProduct
                          }
                        }
                        ... on ProductDeleted {
                          product {
                            ... BaseProduct
                          }
                        }
                      }
                    }

                    fragment BaseProduct on Product {
                      id
                      slug
                      name
                      category {
                        slug
                      }
                    }
                    "#,
                )
                .add_async_events(vec![
                    AsyncWebhookEventType::ProductCreated,
                    AsyncWebhookEventType::ProductUpdated,
                    AsyncWebhookEventType::ProductDeleted,
                ])
                .build(),
        )
        .add_permission(AppPermission::ManageProducts)
        .build();
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
