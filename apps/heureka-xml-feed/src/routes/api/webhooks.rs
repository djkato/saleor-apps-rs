use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
};
use saleor_app_sdk::{
    headers::{SALEOR_API_URL_HEADER, SALEOR_EVENT_HEADER},
    webhooks::{
        AsyncWebhookEventType,
        utils::{EitherWebhookType, get_webhook_event_type},
    },
};

use tracing::{debug, info, trace};

use crate::{
    app::AppState,
    error_template::AxumError,
    queries::products_variants_categories::{
        CategoryCreated, CategoryDeleted, CategoryUpdated, ProductCreated, ProductDeleted,
        ProductUpdated, ProductVariantCreated, ProductVariantDeleted, ProductVariantUpdated,
        ShippingZoneCreated, ShippingZoneDeleted, ShippingZoneUpdated,
    },
    server::event_handler::Event,
};

pub async fn webhooks(
    headers: HeaderMap,
    State(state): State<AppState>,
    //Will try to convert req body to ProductUpdated type, else returns 400
    data: String,
) -> Result<StatusCode, AxumError> {
    debug!("/api/webhooks");
    trace!("req: {:?}", &data);
    trace!("headers: {:?}", headers);

    let url = headers
        .get(SALEOR_API_URL_HEADER)
        .ok_or(AxumError::MissingHeader(SALEOR_API_URL_HEADER.to_owned()))?
        .to_str()?
        .to_owned();
    if url != state.settings.allowed_host {
        debug!("webhook didn't come from allowed host");
        return Ok(StatusCode::METHOD_NOT_ALLOWED);
    }

    let event_type = get_webhook_event_type(&headers)
        .map_err(|_| AxumError::MissingHeader(SALEOR_EVENT_HEADER.to_owned()))?;
    if let EitherWebhookType::Async(a) = event_type {
        // TODO: Extract this into a function so You can check what the error was if something fails
        match a {
            AsyncWebhookEventType::ProductUpdated => {
                let product: ProductUpdated = serde_json::from_str(&data)?;
                state
                    .task_queue_sender
                    .send(Event::ProductUpdated(product))
                    .await?;
            }
            AsyncWebhookEventType::ProductCreated => {
                let product: ProductCreated = serde_json::from_str(&data)?;
                state
                    .task_queue_sender
                    .send(Event::ProductCreated(product))
                    .await?;
            }
            AsyncWebhookEventType::ProductDeleted => {
                let product: ProductDeleted = serde_json::from_str(&data)?;
                state
                    .task_queue_sender
                    .send(Event::ProductDeleted(product))
                    .await?;
            }
            AsyncWebhookEventType::CategoryCreated => {
                let category: CategoryCreated = serde_json::from_str(&data)?;
                state
                    .task_queue_sender
                    .send(Event::CategoryCreated(category))
                    .await?;
            }
            AsyncWebhookEventType::CategoryUpdated => {
                let category: CategoryUpdated = serde_json::from_str(&data)?;
                state
                    .task_queue_sender
                    .send(Event::CategoryUpdated(category))
                    .await?;
            }
            AsyncWebhookEventType::CategoryDeleted => {
                let category: CategoryDeleted = serde_json::from_str(&data)?;
                state
                    .task_queue_sender
                    .send(Event::CategoryDeleted(category))
                    .await?;
            }
            AsyncWebhookEventType::ProductVariantCreated => {
                let variant: ProductVariantCreated = serde_json::from_str(&data)?;
                state
                    .task_queue_sender
                    .send(Event::ProductVariantCreated(variant))
                    .await?;
            }
            AsyncWebhookEventType::ProductVariantUpdated => {
                let variant: ProductVariantUpdated = serde_json::from_str(&data)?;
                state
                    .task_queue_sender
                    .send(Event::ProductVariantUpdated(variant))
                    .await?;
            }
            AsyncWebhookEventType::ProductVariantDeleted => {
                let variant: ProductVariantDeleted = serde_json::from_str(&data)?;
                state
                    .task_queue_sender
                    .send(Event::ProductVariantDeleted(variant))
                    .await?;
            }
            AsyncWebhookEventType::ShippingZoneCreated => {
                let shipping_zone: ShippingZoneCreated = serde_json::from_str(&data)?;
                state
                    .task_queue_sender
                    .send(Event::ShippingZoneCreated(shipping_zone))
                    .await?;
            }
            AsyncWebhookEventType::ShippingZoneUpdated => {
                let shipping_zone: ShippingZoneUpdated = serde_json::from_str(&data)?;
                state
                    .task_queue_sender
                    .send(Event::ShippingZoneUpdated(shipping_zone))
                    .await?;
            }
            AsyncWebhookEventType::ShippingZoneDeleted => {
                let shipping_zone: ShippingZoneDeleted = serde_json::from_str(&data)?;
                state
                    .task_queue_sender
                    .send(Event::ShippingZoneDeleted(shipping_zone))
                    .await?;
            }
            _ => (),
        }
    }

    info!("webhook proccessed");
    Ok(StatusCode::OK)
}
