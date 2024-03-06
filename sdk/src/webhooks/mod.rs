pub mod utils;

use serde::{Deserialize, Serialize};
use strum_macros::EnumString;

use crate::config::Config;

#[derive(Debug, Clone, Serialize, Deserialize, EnumString)]
//kinda annoying that in an apps manifest, the `AsyncWebhookEventType` is in SCREAMING_SNAKE_CASE,
//but when receiving saleors webhook the header `saleor-event` is in snake_case,
//have to serialize and deserialize the enum two different ways
#[serde(rename_all(deserialize = "snake_case", serialize = "SCREAMING_SNAKE_CASE"))]
#[strum(serialize_all = "snake_case")]
pub enum AsyncWebhookEventType {
    AnyEvents,
    AccountConfirmationRequested,
    AccountChangeEmailRequested,
    AccountEmailChanged,
    AccountSetPasswordRequested,
    AccountConfirmed,
    AccountDeleteRequested,
    AccountDeleted,
    AddressCreated,
    AddressUpdated,
    AddressDeleted,
    AppInstalled,
    AppUpdated,
    AppDeleted,
    AppStatusChanged,
    AttributeCreated,
    AttributeUpdated,
    AttributeDeleted,
    AttributeValueCreated,
    AttributeValueUpdated,
    AttributeValueDeleted,
    CategoryCreated,
    CategoryUpdated,
    CategoryDeleted,
    ChannelCreated,
    ChannelUpdated,
    ChannelDeleted,
    ChannelStatusChanged,
    ChannelMetadataUpdated,
    GiftCardCreated,
    GiftCardUpdated,
    GiftCardDeleted,
    GiftCardSent,
    GiftCardStatusChanged,
    GiftCardMetadataUpdated,
    GiftCardExportCompleted,
    MenuCreated,
    MenuUpdated,
    MenuDeleted,
    MenuItemCreated,
    MenuItemUpdated,
    MenuItemDeleted,
    OrderCreated,
    OrderConfirmed,
    OrderPaid,
    OrderFullyPaid,
    OrderRefunded,
    OrderFullyRefunded,
    OrderUpdated,
    OrderCancelled,
    OrderExpired,
    OrderFulfilled,
    OrderMetadataUpdated,
    OrderBulkCreated,
    FulfillmentCreated,
    FulfillmentCanceled,
    FulfillmentApproved,
    FulfillmentMetadataUpdated,
    FulfillmentTrackingNumberUpdated,
    DraftOrderCreated,
    DraftOrderUpdated,
    DraftOrderDeleted,
    SaleCreated,
    SaleUpdated,
    SaleDeleted,
    SaleToggle,
    PromotionCreated,
    PromotionUpdated,
    PromotionDeleted,
    PromotionStarted,
    PromotionEnded,
    PromotionRuleCreated,
    PromotionRuleUpdated,
    PromotionRuleDeleted,
    InvoiceRequested,
    InvoiceDeleted,
    InvoiceSent,
    CustomerCreated,
    CustomerUpdated,
    CustomerDeleted,
    CustomerMetadataUpdated,
    CollectionCreated,
    CollectionUpdated,
    CollectionDeleted,
    CollectionMetadataUpdated,
    ProductCreated,
    ProductUpdated,
    ProductDeleted,
    ProductMetadataUpdated,
    ProductExportCompleted,
    ProductMediaCreated,
    ProductMediaUpdated,
    ProductMediaDeleted,
    ProductVariantCreated,
    ProductVariantUpdated,
    ProductVariantDeleted,
    ProductVariantMetadataUpdated,
    ProductVariantOutOfStock,
    ProductVariantBackInStock,
    ProductVariantStockUpdated,
    CheckoutCreated,
    CheckoutUpdated,
    CheckoutFullyPaid,
    CheckoutMetadataUpdated,
    NotifyUser,
    PageCreated,
    PageUpdated,
    PageDeleted,
    PageTypeCreated,
    PageTypeUpdated,
    PageTypeDeleted,
    PermissionGroupCreated,
    PermissionGroupUpdated,
    PermissionGroupDeleted,
    ShippingPriceCreated,
    ShippingPriceUpdated,
    ShippingPriceDeleted,
    ShippingZoneCreated,
    ShippingZoneUpdated,
    ShippingZoneDeleted,
    ShippingZoneMetadataUpdated,
    StaffCreated,
    StaffUpdated,
    StaffDeleted,
    StaffSetPasswordRequested,
    TransactionItemMetadataUpdated,
    TranslationCreated,
    TranslationUpdated,
    WarehouseCreated,
    WarehouseUpdated,
    WarehouseDeleted,
    WarehouseMetadataUpdated,
    VoucherCreated,
    VoucherUpdated,
    VoucherDeleted,
    VoucherMetadataUpdated,
    VoucherCodeExportCompleted,
    Observability,
    ThumbnailCreated,
    ShopMetadataUpdated,
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumString)]
//kinda annoying that in an apps manifest, the `AsyncWebhookEventType` is in SCREAMING_SNAKE_CASE,
//but when receiving saleors webhook the header `saleor-event` is in snake_case,
//have to serialize and deserialize the enum two different ways
#[serde(rename_all(deserialize = "snake_case", serialize = "SCREAMING_SNAKE_CASE"))]
#[strum(serialize_all = "snake_case")]
pub enum SyncWebhookEventType {
    PaymentListGateways,
    PaymentAuthorize,
    PaymentCapture,
    PaymentRefund,
    PaymentVoid,
    PaymentConfirm,
    PaymentProcess,
    CheckoutCalculateTaxes,
    OrderCalculateTaxes,
    TransactionChargeRequested,
    TransactionRefundRequested,
    TransactionCancelationRequested,
    ShippingListMethodsForCheckout,
    CheckoutFilterShippingMethods,
    OrderFilterShippingMethods,
    PaymentGatewayInitializeSession,
    TransactionInitializeSession,
    TransactionProcessSession,
    ListStoredPaymentMethods,
    StoredPaymentMethodDeleteRequested,
    PaymentGatewayInitializeTokenizationSession,
    PaymentMethodInitializeTokenizationSession,
    PaymentMethodProcessTokenizationSession,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WebhookManifest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub async_events: Option<Vec<AsyncWebhookEventType>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync_events: Option<Vec<SyncWebhookEventType>>,
    /**
     * Query is required for a subscription.
     * If you don't need a payload, you can provide empty query like this:
     *
     * subscription {
     *   event {
     *     __typename
     *   }
     * }
     */
    pub query: String,
    /** The full URL of the endpoint where request will be sent */
    pub target_url: String,
    pub is_active: Option<bool>,
}

#[derive(Default)]
pub struct WebhookManifestBuilder {
    pub webhook_manifest: WebhookManifest,
}

impl WebhookManifestBuilder {
    pub fn set_name(mut self, name: &str) -> Self {
        self.webhook_manifest.name = name.to_owned();
        self
    }
    pub fn set_query(mut self, query: &str) -> Self {
        self.webhook_manifest.query = query.to_owned();
        self
    }
    pub fn add_async_event(mut self, async_event: AsyncWebhookEventType) -> Self {
        if let Some(curr_events) = &mut self.webhook_manifest.async_events {
            curr_events.push(async_event);
        } else {
            self.webhook_manifest.async_events = Some(vec![async_event]);
        }
        self
    }
    pub fn add_async_events(mut self, mut async_events: Vec<AsyncWebhookEventType>) -> Self {
        if let Some(curr_events) = &mut self.webhook_manifest.async_events {
            curr_events.append(&mut async_events);
        } else {
            self.webhook_manifest.async_events = Some(async_events);
        }
        self
    }
    pub fn add_sync_event(mut self, sync_event: SyncWebhookEventType) -> Self {
        if let Some(curr_events) = &mut self.webhook_manifest.sync_events {
            curr_events.push(sync_event);
        } else {
            self.webhook_manifest.sync_events = Some(vec![sync_event]);
        }
        self
    }
    pub fn add_sync_events(mut self, mut sync_events: Vec<SyncWebhookEventType>) -> Self {
        if let Some(curr_events) = &mut self.webhook_manifest.sync_events {
            curr_events.append(&mut sync_events);
        } else {
            self.webhook_manifest.sync_events = Some(sync_events);
        }
        self
    }
    pub fn set_target_url(mut self, url: &str) -> Self {
        self.webhook_manifest.target_url = url.to_owned();
        self
    }
    pub fn set_is_active(mut self, active: bool) -> Self {
        self.webhook_manifest.is_active = Some(active);
        self
    }
    pub fn build(self) -> WebhookManifest {
        self.webhook_manifest
    }
}

impl WebhookManifest {
    /**
     * Creates defaults of name(<cargo_app_name> webhook) and target url(/api/webhooks) from config and env.
     */
    pub fn new(config: &Config) -> WebhookManifestBuilder {
        WebhookManifestBuilder {
            webhook_manifest: WebhookManifest {
                target_url: format!("{}/api/webhooks", config.app_api_base_url),
                name: env!("CARGO_PKG_NAME").to_owned() + " webhook",
                is_active: Some(true),
                ..Default::default()
            },
        }
    }
}
