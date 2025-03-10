use serde::{Deserialize, Serialize};
pub mod extension;
use strum_macros::EnumString;
use thiserror::Error;

use crate::{config::Config, webhooks::WebhookManifest};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    ManageOrdersImport,
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

#[derive(Debug, Clone, Serialize, Deserialize, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum LocaleCode {
    Ar,
    Az,
    Bg,
    Bn,
    Ca,
    Cs,
    Da,
    De,
    El,
    En,
    Es,
    EsCO,
    Et,
    Fa,
    Fr,
    Hi,
    Hu,
    Hy,
    Id,
    Is,
    It,
    Ja,
    Ko,
    Mn,
    Nb,
    Nl,
    Pl,
    Pt,
    PtBR,
    Ro,
    Ru,
    Sk,
    Sl,
    Sq,
    Sr,
    Sv,
    Th,
    Tr,
    Uk,
    Vi,
    ZhHans,
    ZhHant,
}

impl Default for LocaleCode {
    fn default() -> Self {
        Self::En
    }
}
impl Default for AppExtensionMount {
    fn default() -> Self {
        Self::ProductOverviewMoreActions
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AppExtensionTarget {
    Popup,
    AppPage,
}
impl Default for AppExtensionTarget {
    fn default() -> Self {
        Self::Popup
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
     Allows app installation for specific Saleor versions, using semver.
     https://github.com/npm/node-semver#versions

     If not set, Saleor will allow installation for every version

     In Saleor versions lower than 3.13, this field will be ignored

     Examples:
     ">=3.10" - allow for versions 3.10 or newer
     ">=3.10 <4" - allow for versions 3.10 and newer, but not 4.0 and newer
     ">=3.10 <4 || 4.0.0" - 3.10 and newer, less than 4, but allow exactly 4.0.0
    */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_saleor_version: Option<String>,
    /**
     App author name displayed in the dashboard

     In Saleor versions lower than 3.13, this field will be ignored
    */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /**
     Add brand-specific metadata to the app

     Available from Saleor 3.15. In previous versions will be ignored
    */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brand: Option<SaleorAppBranding>,
}

pub struct AppManifestBuilder {
    pub manifest: AppManifest,
}

impl AppManifestBuilder {
    /**
    to simply create a webhook manifest, you can use WebhookManifest::new()
    */
    pub fn add_webhook(mut self, webhook: WebhookManifest) -> Self {
        if let Some(webhooks) = &mut self.manifest.webhooks {
            webhooks.push(webhook)
        } else {
            self.manifest.webhooks = Some(vec![webhook]);
        }
        self
    }
    pub fn add_permission(mut self, permissions: AppPermission) -> Self {
        self.manifest.permissions.push(permissions);
        self
    }
    pub fn add_permissions(mut self, mut permissions: Vec<AppPermission>) -> Self {
        self.manifest.permissions.append(&mut permissions);
        self
    }
    pub fn add_extension(mut self, extension: AppExtension) -> Self {
        match &mut self.manifest.extensions {
            Some(e) => e.push(extension),
            None => self.manifest.extensions = Some(vec![extension]),
        }
        self
    }
    pub fn add_extensions(mut self, mut extensions: Vec<AppExtension>) -> Self {
        match &mut self.manifest.extensions {
            Some(e) => e.append(&mut extensions),
            None => self.manifest.extensions = Some(extensions),
        }
        self
    }
    pub fn build(self) -> Result<AppManifest, AppManifestBuilderError> {
        if let Some(ext) = self
            .manifest
            .extensions
            .as_ref()
            .map(|exts| exts.iter().flat_map(|e| &e.permissions).collect::<Vec<_>>())
        {
            match ext.iter().all(|e| self.manifest.permissions.contains(e)) {
                true => Ok(self.manifest),
                false => Err(AppManifestBuilderError::MismatchedPermissions),
            }
        } else {
            Ok(self.manifest)
        }
    }
}
#[derive(Error, Debug)]
pub enum AppManifestBuilderError {
    #[error("manifest.permissions doesn't list all permissions needed in manifest.extensions.permissions")]
    MismatchedPermissions,
}
pub struct CargoInfo {
    pub name: String,
    pub description: String,
    pub authors: String,
    pub version: String,
    pub homepage: String,
}

#[macro_export]
macro_rules! cargo_info {
    () => {
        saleor_app_sdk::manifest::CargoInfo {
            name: env!("CARGO_PKG_NAME").to_owned(),
            description: env!("CARGO_PKG_DESCRIPTION").to_owned(),
            authors: env!("CARGO_PKG_AUTHORS").to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            homepage: env!("CARGO_PKG_HOMEPAGE").to_owned(),
        }
    };
}

pub use cargo_info;
impl AppManifestBuilder {
    /**
     Builder for AppManifest

     Takes these out of config:
     - Takes fields id, saleor_version, logo, token_target_url

     And these out of the environment:
     - name(CARGO_PKG_NAME), about(CARGO_PKG_DESCRIPTION), author(CARGO_PKG_AUTHORS),

     version(CARGO_PKG_VERSION), homepage_url(CARGO_PKG_HOMEPAGE)

     To set webhooks and permissions use the add_webhook() and add_permissions()

    */
    pub fn new(config: &Config, info: CargoInfo) -> Self {
        AppManifestBuilder {
            manifest: AppManifest {
                id: info.name.clone(),
                required_saleor_version: Some(config.required_saleor_version.clone()),
                name: info.name,
                about: Some(info.description),
                author: Some(info.authors),
                version: info.version,
                app_url: config.app_api_base_url.clone(),
                configuration_url: Some(config.app_api_base_url.clone()),
                token_target_url: format!("{}/api/register", config.app_api_base_url.clone()),
                permissions: vec![],
                homepage_url: Some(info.homepage.clone()),
                data_privacy_url: Some(info.homepage.clone()),
                support_url: Some(info.homepage.clone()),
                brand: Some(SaleorAppBranding {
                    logo: SaleorAppBrandingDefault {
                        default: format!("{}/logo.png", config.app_api_base_url),
                    },
                }),
                ..Default::default()
            },
        }
    }
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
