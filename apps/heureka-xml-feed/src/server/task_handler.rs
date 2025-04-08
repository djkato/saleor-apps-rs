use crate::{
    app::{AppSettings, AppState},
    queries::event_products_updated::{
        Category, CategoryCreated, CategoryDeleted, CategoryUpdated, Product2, ProductCreated, ProductDeleted, ProductUpdated, ProductVariant, ProductVariantCreated, ProductVariantDeleted, ProductVariantUpdated, ShippingZone, ShippingZoneCreated, ShippingZoneDeleted, ShippingZoneUpdated
    },
};
use tokio::{sync::mpsc::Receiver, task::JoinHandle};
use tracing::{debug,  info, warn};

pub struct EventHandler {
    receiver: Receiver<Event>,
    settings: AppSettings,
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
    Unknown,
}

#[derive(Debug, Clone)]
pub struct RegenerateEvent {
    pub state: AppState,
    pub saleor_api_url: String,
}

impl EventHandler {
    pub fn start(settings: AppSettings, receiver: Receiver<Event>) -> JoinHandle<()> {
        let s = Self { settings, receiver };
        tokio::spawn(s.listen())
    }

    async fn listen(mut self) {
        while let Some(message) = self.receiver.recv().await {
            debug!("received Event: {:?}", &message);
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
                        self.any_deleted(&product.id,AnyDeletedType::Product).await;
                    } else {
                        warn!("Event::ProductDeleted missing data");
                    }
                }

                Event::CategoryCreated(category_created) => {
                    if let Some(category) = category_created.clone().category {
                        self.category_updated_or_created(category)
                            .await;
                    } else {
                        warn!("Event::CategoryCreated/Updated missing data");
                    }
                }
                Event::CategoryUpdated(category_updated) => {
                    if let Some(category) = category_updated.clone().category {
                        self.category_updated_or_created( category)
                            .await;
                    } else {
                        warn!("Event::CategoryCreated/Updated missing data");
                    }
                }
                Event::CategoryDeleted(category) => {
                    if let Some(category) = category.category {
                        self.any_deleted(&category.id, AnyDeletedType::Category).await;
                    } else {
                        warn!("Event::CategoryDeleted missing data");
                    }
                }
                Event::ProductVariantCreated(variant) => {
                    if let Some(variant) = variant.product_variant {
                        self.variant_updated_or_created (variant).await;
                    }
                }
                Event::ProductVariantUpdated(variant) => {
                    if let Some(variant) = variant.product_variant {
                        self.variant_updated_or_created (variant).await;
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
                        self.any_deleted(&shipping_zone.id, AnyDeletedType::ShippingZone).await;
                    }
                }

                Event::Regenerate(r) => (),
                // match regenerate(r.state, r.saleor_api_url).await {
                //     Ok(_) => info!("regenerate: Fully created sitemap!"),
                //     Err(e) => error!("regenerate: ERR! {:?}", e),
                // },
                Event::Unknown => (),
            }
            info!("Event succesfully handled");
        }
    }
}

/* =============== Event handlers =============== */
impl EventHandler {
    async fn product_updated_or_created(&self, data: Product2) {
        info!("called!")
    }

    async fn variant_updated_or_created (&self, data: ProductVariant) {
        info!("called!")
    }

    async fn any_deleted(&self, id: &cynic::Id, typ:AnyDeletedType ) {
        info!("called!")
    }

    async fn category_updated_or_created(&self, data:Category) { 
        info!("called!")
    }
    async fn shipping_zone_updated_or_created(&self, data: ShippingZone) {
        info!("called!")
    }
}

enum AnyDeletedType {
    Product,
    Category,
    Variant,
    ShippingZone
}


