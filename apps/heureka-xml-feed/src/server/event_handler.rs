use crate::{
    app::{AppSettings, AppState},
    queries::{
        products_variants_categories::{
            Category, CategoryCreated, CategoryDeleted, CategoryUpdated, Product, ProductCreated,
            ProductDeleted, ProductUpdated, ProductVariant, ProductVariant2, ProductVariantCreated,
            ProductVariantDeleted, ProductVariantUpdated, ShippingZoneCreated, ShippingZoneDeleted,
            ShippingZoneUpdated,
        },
        query_shipping_details::ShippingZone,
        surreal_types,
    },
    server::{
        VariantUrlTemplateContext, find_category_text,
        graphqls::{get_all_products, get_shipping_zones},
        surrealdbs::{
            get_product_related_categories, get_product_related_variants, get_products,
            save_product_categories_on_regenerate, save_shipping_zone_to_db, save_variants_to_db,
        },
        try_create_shopitem, variant_url_from_template,
    },
};
use cynic::GraphQlError;
use heureka_xml_feed::{Shop, ShopItem};
use saleor_app_sdk::apl::AplError;
use surrealdb::{Surreal, engine::any::Any};
use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};
use tracing::{debug, error, info, warn};

use super::TryIntoShopItemError;

pub struct EventHandler {
    receiver: Receiver<Event>,
    settings: AppSettings,
    db_handle: Surreal<Any>,
}

#[derive(Debug, Clone)]
pub enum Event {
    ProductUpdated(ProductUpdated),
    ProductCreated(ProductCreated),
    ProductDeleted(ProductDeleted),
    CategoryCreated(CategoryCreated),
    CategoryUpdated(CategoryUpdated),
    CategoryDeleted(CategoryDeleted),
    ProductVariantCreated(ProductVariantCreated),
    ProductVariantUpdated(ProductVariantUpdated),
    ProductVariantDeleted(ProductVariantDeleted),
    ShippingZoneCreated(ShippingZoneCreated),
    ShippingZoneUpdated(ShippingZoneUpdated),
    ShippingZoneDeleted(ShippingZoneDeleted),
    Regenerate(RegenerateEvent),
    CreateXML(CreateXMLEvent),
    Unknown,
}

#[derive(Debug, Clone)]
pub struct CreateXMLEvent {
    pub state: AppState,
    pub sender: Sender<Option<String>>,
}

#[derive(Debug, Clone)]
pub struct RegenerateEvent {
    pub state: AppState,
    pub saleor_api_url: String,
}

impl EventHandler {
    pub fn start(
        settings: AppSettings,
        receiver: Receiver<Event>,
        db_handle: Surreal<Any>,
    ) -> JoinHandle<()> {
        let s = Self {
            settings,
            receiver,
            db_handle,
        };
        tokio::spawn(s.listen())
    }

    async fn listen(mut self) {
        while let Some(message) = self.receiver.recv().await {
            // debug!("received Event: {:?}", &message);
            match message {
                Event::ProductCreated(product_created) => {
                    if let Some(product) = product_created.clone().product {
                        self.product_updated_or_created(product).await;
                    } else {
                        warn!("Event::ProductCreated/Updated missing data");
                    }
                }
                Event::ProductUpdated(product_updated) => {
                    if let Some(product) = product_updated.clone().product {
                        self.product_updated_or_created(product).await;
                    } else {
                        warn!("Event::ProductCreated/Updated missing data");
                    }
                }
                Event::ProductDeleted(product) => {
                    if let Some(product) = product.product {
                        self.any_deleted(&product.id, AnyDeletedType::Product).await;
                    } else {
                        warn!("Event::ProductDeleted missing data");
                    }
                }
                Event::CategoryCreated(category_created) => {
                    if let Some(category) = category_created.clone().category {
                        self.category_updated_or_created(category).await;
                    } else {
                        warn!("Event::CategoryCreated/Updated missing data");
                    }
                }
                Event::CategoryUpdated(category_updated) => {
                    if let Some(category) = category_updated.clone().category {
                        self.category_updated_or_created(category).await;
                    } else {
                        warn!("Event::CategoryCreated/Updated missing data");
                    }
                }
                Event::CategoryDeleted(category) => {
                    if let Some(category) = category.category {
                        self.any_deleted(&category.id, AnyDeletedType::Category)
                            .await;
                    } else {
                        warn!("Event::CategoryDeleted missing data");
                    }
                }
                Event::ProductVariantCreated(variant) => {
                    if let Some(variant) = variant.product_variant {
                        self.variant_updated_or_created(variant).await;
                    }
                }
                Event::ProductVariantUpdated(variant) => {
                    if let Some(variant) = variant.product_variant {
                        self.variant_updated_or_created(variant).await;
                    }
                }
                Event::ProductVariantDeleted(variant) => {
                    if let Some(variant) = variant.product_variant {
                        self.any_deleted(&variant.id, AnyDeletedType::Variant).await;
                    }
                }
                Event::ShippingZoneCreated(shipping_zone) => {
                    if let Some(shipping_zone) = shipping_zone.shipping_zone {
                        self.shipping_zone_updated_or_created(shipping_zone).await;
                    }
                }
                Event::ShippingZoneUpdated(shipping_zone) => {
                    if let Some(shipping_zone) = shipping_zone.shipping_zone {
                        self.shipping_zone_updated_or_created(shipping_zone).await;
                    }
                }
                Event::ShippingZoneDeleted(shipping_zone) => {
                    if let Some(shipping_zone) = shipping_zone.shipping_zone {
                        self.any_deleted(&shipping_zone.id, AnyDeletedType::ShippingZone)
                            .await;
                    }
                }
                Event::Regenerate(r) => {
                    if let Err(e) = self.regenerate(r).await {
                        error!("{:?}", e);
                    };
                }
                Event::CreateXML(ev) => {
                    _ = match self.db_to_xml().await {
                        Ok(xml) => ev.sender.send(Some(xml)).await,
                        Err(e) => {
                            error!("{:?}", e);
                            ev.sender.send(None).await
                        }
                    }
                }
                Event::Unknown => (),
            }
            info!("Event handled");
        }
    }
}

/* =============== Event handlers =============== */
#[derive(thiserror::Error, Debug)]
pub enum EventHandlerError {
    ///(Product, Variant/Category)
    #[error("Product is missing an important relation, like category or variant on product, {0:?}")]
    ProductMissingRelation(MissingRelation),
    #[error("Issue during graphql operation, {0}")]
    GraphQl(#[from] GraphQlError),
    #[error("Error fetching APL token, {0}")]
    Apl(#[from] AplError),
    #[error("Error during graphql operation, {0}")]
    Surf(surf::Error),
    #[error("Failed surrealdb query: {0}")]
    SurrealDB(#[from] surrealdb::Error),
    #[error("Failed surrealdb query: {0}")]
    HeurekaXml(#[from] heureka_xml_feed::Error),
    #[error("Failed turning product into ShopItem, {0}")]
    TryIntoShopItem(#[from] TryIntoShopItemError),
    #[error("Shipping zones misconfigured, {0}")]
    ShippingZoneMisconfigured(String),
    #[error(
        "Product doesn't have a categorytext (products category and all it's parent categories have missing 'heureka_categorytext' metadata field), {0}"
    )]
    ProductMissingCategoryText(String),

    #[error("failed converting description from json to string, {0}")]
    SerdeJson(#[from] serde_json::Error),
}

impl From<surf::Error> for EventHandlerError {
    fn from(err: surf::Error) -> Self {
        EventHandlerError::Surf(err)
    }
}

#[derive(Debug, Clone)]
pub enum MissingRelation {
    Variant,
    Category,
}

impl EventHandler {
    async fn regenerate(&mut self, ev: RegenerateEvent) -> Result<(), Vec<EventHandlerError>> {
        debug!("starting database regeneration (querying all products)");
        let mut errors: Vec<EventHandlerError> = vec![];

        let token = ev
            .state
            .saleor_app
            .lock()
            .await
            .apl
            .get(&ev.saleor_api_url)
            .await
            .map_err(|e| vec![e.into()])?;

        let shipping_zones = get_shipping_zones(
            &ev.saleor_api_url,
            &token.token,
            &self.settings.channel_slug,
            &self.settings.shipping_zone_ids,
        )
        .await
        .map_err(|e| vec![e])?;

        debug!("collected {} shipping zones", shipping_zones.len());

        if shipping_zones.is_empty() {
            error!("No shipping zones collected, misconfigured env?");
            return Err(vec![EventHandlerError::ShippingZoneMisconfigured(
                "No shipping zones collected from graphql".to_owned(),
            )]);
        }

        let all_products = get_all_products(
            &ev.saleor_api_url,
            &self.settings.channel_slug,
            &token.token,
        )
        .await
        .map_err(|e| vec![e])?;

        /* SAVE THEM TO DB */
        let db = &mut self.db_handle;

        for zone in shipping_zones {
            save_shipping_zone_to_db(&zone, db).await.unwrap();
            if let Err(e) = save_shipping_zone_to_db(&zone, db).await {
                errors.push(e);
            }
        }

        for product in all_products {
            if let Err(e) = save_product_categories_on_regenerate(&product, &ev, &token, db).await {
                errors.push(e);
            }

            for variant in product.clone().variants {
                save_variants_to_db(&variant, db, &product).await.unwrap();
                if let Err(e) = save_variants_to_db(&variant, db, &product).await {
                    errors.push(e);
                }
            }
        }

        debug!("Database regenerated!");
        debug!("Validating XML from DB");

        let xml = self.db_to_xml().await;
        match xml {
            Err(mut e) => match e.is_empty() {
                true => Ok(()),
                false => {
                    error!("Errors during db_to_xml(), {:?}", &e);
                    errors.append(&mut e);
                    Err(errors)
                }
            },
            Ok(_) => Ok(()),
        }
    }

    async fn product_updated_or_created(&self, _data: Product) {
        info!("called!")
    }

    async fn variant_updated_or_created(&self, _data: ProductVariant) {
        info!("called!")
    }

    async fn any_deleted(&self, _id: &cynic::Id, _typ: AnyDeletedType) {
        info!("called!")
    }

    async fn category_updated_or_created(&self, _data: Category) {
        info!("called!")
    }

    async fn shipping_zone_updated_or_created(&self, _data: ShippingZone) {
        info!("called!")
    }

    async fn db_to_xml(&mut self) -> Result<String, Vec<EventHandlerError>> {
        debug!("Creating XML from DB data");
        let mut errors = vec![];

        let db = &mut self.db_handle;
        debug!("Collecting DB products");
        let products: Vec<surreal_types::Product> = get_products(db).await.map_err(|e| vec![e])?;

        let mut shopitems: Vec<ShopItem> = vec![];

        debug!("Collecting DB shipping zones");
        let shipping_zones = super::surrealdbs::get_shipping_zones(db)
            .await
            .map_err(|e| vec![e])?;

        if shipping_zones.is_empty() {
            return Err(vec![EventHandlerError::ShippingZoneMisconfigured(
                "No shipping zones present in db".to_owned(),
            )]);
        }

        for product in products {
            debug!(
                "Collecting DB variants for product {}:{}",
                &product.name,
                &product.id.to_string()
            );
            let variants: Vec<surreal_types::ProductVariant> =
                match get_product_related_variants(db, &product).await {
                    Ok(variants) => match variants.is_empty() {
                        false => variants,
                        true => {
                            errors.push(EventHandlerError::ProductMissingRelation(
                                MissingRelation::Variant,
                            ));
                            continue;
                        }
                    },
                    Err(e) => {
                        warn!(
                            "failed getting variants related to product {}:{}, skipping it.\n {e}",
                            &product.name,
                            &product.id.to_string()
                        );
                        errors.push(e);
                        continue;
                    }
                };

            let categories = match get_product_related_categories(db, &product).await {
                Ok(categories) => match categories.is_empty() {
                    false => categories,
                    true => {
                        errors.push(EventHandlerError::ProductMissingRelation(
                            MissingRelation::Category,
                        ));
                        continue;
                    }
                },
                Err(e) => {
                    warn!(
                        "failed getting categories related to product {}:{}, skipping it.\n {e}",
                        &product.name,
                        &product.id.to_string()
                    );
                    errors.push(e);
                    continue;
                }
            };

            let categorytext = match find_category_text(&categories) {
                Some(c) => c,
                None => {
                    warn!(
                        "failed finding heureka category text for product {}:{}, skipping it",
                        &product.name,
                        &product.id.to_string()
                    );
                    errors.push(EventHandlerError::ProductMissingCategoryText(
                        product.id.to_string().to_string(),
                    ));
                    continue;
                }
            };

            for variant in variants {
                let mut deliveries = vec![];

                for zone in shipping_zones.clone() {
                    match zone.into_deliveries(
                        variant.clone().get_weight(product.clone()),
                        self.settings.is_shipping_cod,
                        self.settings.shipping_price_cod_extra,
                    ) {
                        Ok(mut d) => deliveries.append(&mut d),
                        Err(e) => errors.push(e),
                    };
                }

                if deliveries.is_empty() {
                    error!("no shipping zones/deliveries are configured",);
                    errors.push(EventHandlerError::ShippingZoneMisconfigured("Not a single delivery zone configured, please configure your shipping settings".to_string()));
                    return Err(errors);
                }

                let variant_url_ctx = VariantUrlTemplateContext {
                    product: &product,
                    variant: &variant,
                    category: categories
                        .first()
                        .expect("Caetgory was checked to have at least one, how did this happen?"),
                };

                let variant_url = match variant_url_from_template(
                    self.settings.variant_url_template.clone(),
                    &variant_url_ctx,
                ) {
                    Ok(u) => u,
                    Err(e) => {
                        warn!(
                            "failed creating for variant {}:{} url from template {} with context {:?}, skipping variant",
                            &variant.name,
                            variant.id.to_string(),
                            self.settings.variant_url_template.clone(),
                            &variant_url_ctx
                        );
                        let e1: TryIntoShopItemError = e.into();
                        errors.push(e1.into());
                        continue;
                    }
                };

                let shopitem = match try_create_shopitem(
                    product.clone(),
                    variant.clone(),
                    deliveries,
                    categorytext.clone(),
                    variant_url,
                    self.settings.tax_rate.clone(),
                ) {
                    Ok(i) => i,
                    Err(e) => {
                        warn!(
                            "failed turning variant {}:{} into heureka shopitem, skipping variant",
                            &variant.name,
                            &variant.id.to_string()
                        );
                        errors.push(e.into());
                        continue;
                    }
                };

                shopitems.push(shopitem);
            }
        }

        let shop = Shop {
            shop_item: shopitems,
        };

        let xml = match shop.try_to_xml() {
            Err(e) => {
                errors.push(e.into());
                String::new()
            }
            Ok(xml) => xml,
        };

        match errors.is_empty() {
            true => Ok(xml),
            false => Err(errors),
        }
    }
}

enum AnyDeletedType {
    Product,
    Category,
    Variant,
    ShippingZone,
}
