use std::time::Duration;

use anyhow::{bail, Result};
use redis::{AsyncCommands, Commands, ConnectionLike, RedisError};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub auth_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AsyncWebhookEventType {
    AccountConfirmationRequested,
    AccountDeleteRequested,
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
    GiftCardCreated,
    GiftCardUpdated,
    GiftCardDeleted,
    GiftCardSent,
    GiftCardStatusChanged,
    GiftCardMetadataUpdated,
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
    DraftOrderCreated,
    DraftOrderUpdated,
    DraftOrderDeleted,
    SaleCreated,
    SaleUpdated,
    SaleDeleted,
    SaleToggle,
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
    ProductMediaCreated,
    ProductMediaUpdated,
    ProductMediaDeleted,
    ProductMetadataUpdated,
    ProductVariantCreated,
    ProductVariantUpdated,
    ProductVariantDeleted,
    ProductVariantOutOfStock,
    ProductVariantBackInStock,
    ProductVariantStockUpdated,
    ProductVariantMetadataUpdated,
    CheckoutCreated,
    CheckoutUpdated,
    CheckoutFullyPaid,
    CheckoutMetadataUpdated,
    FulfillmentCreated,
    FulfillmentCanceled,
    FulfillmentApproved,
    FulfillmentMetadataUpdated,
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
    TransactionActionRequest,
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
    OBSERVABILITY,
    ThumbnailCreated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SyncWebhookEventType {
    CheckoutCalculateTaxes,
    OrderCalculateTaxes,
    ShippingListMethodsForCheckout,
    CheckoutFilterShippingMethods,
    OrderFilterShippingMethods,
    TransactionChargeRequested,
    TransactionRefundRequested,
    TransactionCancelationRequested,
    PaymentGatewayInitializeSession,
    TransactionInitializeSession,
    TransactionProcessSession,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AppPermission {
    ManageUsers,
    ManageStaff,
    ImpersonateUser,
    ManageObservability,
    ManageCheckouts,
    HandleCheckouts,
    HandleTaxes,
    ManageTaxes,
    ManageChannels,
    ManageDiscounts,
    ManageGiftCard,
    ManageMenus,
    ManageOrders,
    ManagePages,
    ManagePageTypesAndAttributes,
    HandlePayments,
    ManagePlugins,
    ManageProducts,
    ManageProductTypesAndAttributes,
    ManageShipping,
    ManageSettings,
    ManageTranslations,
    ManageApps,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AppExtensionMount {
    ProductDetailsMoreActions,
    ProductOverviewCreate,
    ProductOverviewMoreActions,
    NavigationCatalog,
    NavigationOrders,
    NavigationCustomers,
    NavigationDiscounts,
    NavigationTranslations,
    NavigationPages,
    OrderDetailsMoreActions,
    OrderOverviewCreate,
    OrderOverviewMoreActions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AppExtensionTarget {
    Popup,
    AppPage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppExtension {
    /** Name which will be displayed in the dashboard */
    pub label: String,
    /** the place where the extension will be mounted */
    pub mount: AppExtensionMount,
    /** Method of presenting the interface
      `POPUP` will present the interface in a modal overlay
      `APP_PAGE` will navigate to the application page
      @default `POPUP`
    */
    pub target: AppExtensionTarget,
    pub permissions: Vec<AppPermission>,
    /** URL of the view to display,
     you can skip the domain and protocol when target is set to `APP_PAGE`, or when your manifest defines an `appUrl`.

     When target is set to `POPUP`, the url will be used to render an `<iframe>`.
    */
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppManifest {
    /** ID of the application used internally by Saleor */
    pub id: String,
    pub version: String,
    /** App's name displayed in the dashboard */
    pub name: String,
    /** Description of the app displayed in the dashboard */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub about: Option<String>,
    /** Array of permissions requested by the app */
    pub permissions: Vec<AppPermission>,
    /** App website rendered in the dashboard */
    pub app_url: String,
    /** Address to the app configuration page, which is rendered in the dashboard
      @deprecated in Saleor 3.5, use appUrl instead
    */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration_url: Option<String>,
    /** Endpoint used during process of app installation

      @see [Installing an app](https://docs.saleor.io/docs/3.x/developer/extending/apps/installing-apps#installing-an-app)
    */
    pub token_target_url: String,
    /** Short description of privacy policy displayed in the dashboard

      @deprecated in Saleor 3.5, use dataPrivacyUrl instead
    */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_privacy: Option<String>,
    /** URL to the full privacy policy */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_privacy_url: Option<String>,
    /**  External URL to the app homepage */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage_url: Option<String>,
    /** External URL to the page where app users can find support */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub support_url: Option<String>,
    /** List of extensions that will be mounted in Saleor's dashboard

    @see For details, please see the [extension section](https://docs.saleor.io/docs/3.x/developer/extending/apps/extending-dashboard-with-apps#key-concepts)
    */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Vec<AppExtension>>,
    /** List of webhooks that will be set.

    @see For details, please look at [asynchronous webhooks](https://docs.saleor.io/docs/3.x/developer/extending/apps/asynchronous-webhooks),
    [synchronous-webhooks](https://docs.saleor.io/docs/3.x/developer/extending/apps/synchronous-webhooks/key-concepts)
    and [webhooks' subscription](https://docs.saleor.io/docs/3.x/developer/extending/apps/subscription-webhook-payloads)

    Be aware that subscription queries are required in manifest sections
    */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhooks: Option<Vec<WebhookManifest>>,
    /**
     * Allows app installation for specific Saleor versions, using semver.
     * https://github.com/npm/node-semver#versions
     *
     * If not set, Saleor will allow installation for every version
     *
     * In Saleor versions lower than 3.13, this field will be ignored
     *
     * Examples:
     * ">=3.10" - allow for versions 3.10 or newer
     * ">=3.10 <4" - allow for versions 3.10 and newer, but not 4.0 and newer
     * ">=3.10 <4 || 4.0.0" - 3.10 and newer, less than 4, but allow exactly 4.0.0
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_saleor_version: Option<String>,
    /**
     * App author name displayed in the dashboard
     *
     * In Saleor versions lower than 3.13, this field will be ignored
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /**
     * Add brand-specific metadata to the app
     *
     * Available from Saleor 3.15. In previous versions will be ignored
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brand: Option<SaleorAppBranding>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaleorAppBranding {
    pub logo: SaleorAppBrandingDefault,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaleorAppBrandingDefault {
    pub default: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthData {
    pub domain: Option<String>,
    pub token: String,
    pub saleor_api_url: String,
    pub app_id: String,
    pub jwks: Option<String>,
}
impl std::fmt::Display for AuthData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(domain:{}\ntoken:{}\nsaleor_api_url:{}\napp_id:{}\njwks:{})",
            self.domain.clone().unwrap_or_default(),
            self.token,
            self.saleor_api_url,
            self.app_id,
            self.jwks.clone().unwrap_or_default()
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AplType {
    Redis,
}

pub trait APL: Sized + Send + Sync + Clone + std::fmt::Debug {
    async fn get(&self, saleor_api_url: &str) -> Result<AuthData>;
    async fn set(&self, auth_data: AuthData) -> Result<()>;
    async fn delete(&self, saleor_api_url: &str) -> Result<()>;
    async fn get_all(&self) -> Result<Vec<AuthData>>;
    async fn is_ready(&self) -> Result<()>;
    async fn is_configured(&self) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct SaleorApp<A: APL> {
    pub apl: A,
}

#[derive(Debug, Clone)]
pub struct RedisApl {
    pub client: redis::Client,
    pub app_api_base_url: String,
}

impl APL for RedisApl {
    async fn get(&self, saleor_api_url: &str) -> Result<AuthData> {
        debug!(" get()");
        let mut conn = self.client.get_async_connection().await?;
        let val: String = conn.get(self.prepare_key(saleor_api_url)).await?;
        debug!("received {val}");
        let val: AuthData = serde_json::from_str(&val)?;
        info!("sucessful get");
        debug!("parsed {val}");

        Ok(val)
    }
    async fn set(&self, auth_data: AuthData) -> Result<()> {
        debug!("set(), {}", auth_data);
        let mut conn = self.client.get_async_connection().await?;
        conn.set(
            self.prepare_key(&auth_data.saleor_api_url),
            serde_json::to_string(&auth_data)?,
        )
        .await?;
        info!("sucessful set");
        Ok(())
    }
    async fn delete(&self, saleor_api_url: &str) -> Result<()> {
        debug!("delete(), {}", saleor_api_url);
        let mut conn = self.client.get_async_connection().await?;
        let val: String = conn.get_del(self.prepare_key(saleor_api_url)).await?;

        debug!("sucessful delete(), {}", val);
        info!("sucessful del");
        Ok(())
    }
    async fn is_ready(&self) -> Result<()> {
        debug!("is_ready()");
        let mut conn = self.client.get_async_connection().await?;
        let val: String = redis::cmd("INFO")
            .arg("server")
            .query_async(&mut conn)
            .await?;

        debug!("sucessful is_ready(), info: {}", val);
        info!("sucessful is_ready");
        Ok(())
    }
    async fn is_configured(&self) -> Result<()> {
        debug!("is_configured()");
        let mut conn = self.client.get_async_connection().await?;
        let val: String = redis::cmd("INFO")
            .arg("server")
            .query_async(&mut conn)
            .await?;

        debug!("sucessful is_configured(), info: {}", val);
        info!("sucessful is_configured");
        Ok(())
    }
    async fn get_all(&self) -> Result<Vec<AuthData>> {
        anyhow::bail!("Redis doens't support getall")
    }
}

impl RedisApl {
    pub fn new(redis_url: String, app_api_base_url: String) -> Result<Self> {
        let client = redis::Client::open(redis_url)?;
        let mut conn = client.get_connection_with_timeout(Duration::from_secs(3))?;
        let val: Result<String, redis::RedisError> =
            redis::cmd("INFO").arg("server").query(&mut conn);

        match val {
            Ok(_) => Ok(Self {
                client,
                app_api_base_url,
            }),
            Err(e) => bail!("failed redis connection, {:?}", e),
        }
    }
    pub fn prepare_key(&self, saleor_api_url: &str) -> String {
        let key = format!("{}:{saleor_api_url}", self.app_api_base_url);
        debug!("made key:'{}'", key);
        key
    }
}
