use axum::{
    extract::{Json, State},
    http::{HeaderMap, StatusCode},
};
use cynic::{http::SurfExt, MutationBuilder};
use saleor_app_sdk::{
    headers::{SALEOR_API_URL_HEADER, SALEOR_EVENT_HEADER},
    webhooks::{
        utils::{get_webhook_event_type, EitherWebhookType},
        AsyncWebhookEventType,
    },
};

use tracing::{debug, info};

use crate::{
    app::AppState,
    error_template::AxumError,
    queries::{
        event_products_updated::ProductUpdated,
        product_metadata_update::{
            MetadataInput, UpdateProductMetadata, UpdateProductMetadataVariables,
        },
    },
};

pub async fn webhooks(
    headers: HeaderMap,
    State(state): State<AppState>,
    //Will try to convert req body to ProductUpdated type, else returns 400
    Json(product): Json<ProductUpdated>,
) -> Result<StatusCode, AxumError> {
    debug!("/api/webhooks");
    debug!("req: {:?}", product);
    debug!("headers: {:?}", headers);

    let url = headers
        .get(SALEOR_API_URL_HEADER)
        .ok_or(AxumError::MissingHeader(SALEOR_API_URL_HEADER.to_owned()))?;
    let event_type = get_webhook_event_type(&headers)
        .map_err(|_| AxumError::MissingHeader(SALEOR_EVENT_HEADER.to_owned()))?;
    if let EitherWebhookType::Async(
        AsyncWebhookEventType::ProductUpdated
        | AsyncWebhookEventType::ProductCreated
        | AsyncWebhookEventType::ProductDeleted,
    ) = event_type
    {
        update_product(product, url.to_str()?, state).await?
    }

    info!("got webhooks!");
    Ok(StatusCode::OK)
}

async fn update_product(
    product: ProductUpdated,
    saleor_api_url: &str,
    state: AppState,
) -> Result<(), anyhow::Error> {
    debug!("Product got changed!");
    if let Some(product) = product.product {
        let operation = UpdateProductMetadata::build(UpdateProductMetadataVariables {
            product_id: &product.id,
            metadata: Some(vec![MetadataInput {
                key: "helloloo",
                value: "hiiiihii",
            }]),
        });
        let saleor_app = state.saleor_app.lock().await;
        let auth_data = saleor_app.apl.get(saleor_api_url).await?;
        let result = surf::post(saleor_api_url)
            .header("Authorization", format!("bearer {}", auth_data.token))
            .run_graphql(operation)
            .await;
        debug!("update product result : {:?}", result);
    }
    Ok(())
}
