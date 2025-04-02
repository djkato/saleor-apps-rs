use serde::Serialize;
use std::{
    fs::{self},
    io::ErrorKind,
};

use crate::{
    app::{AppSettings, AppState},
    queries::event_products_updated::{
        CategoryCreated, CategoryDeleted, CategoryUpdated, Product, Product2, ProductCreated,
        ProductDeleted, ProductUpdated, ProductVariantCreated, ProductVariantDeleted,
        ProductVariantUpdated,
    },
};
use tokio::{sync::mpsc::Receiver, task::JoinHandle};
use tracing::{debug, error, info, warn};

// 10k links google says, but there's also a size limit and my custom params might be messing with
// that? Rather split prematurely to be sure.
const MAX_URL_IN_SET: usize = 50_000;
const DB_FILE_NAME: &str = "db.cbor";
const SITEMAP_FILE_NAME: &str = "sitemap.txt";

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
                        product_updated_or_created(product_created, &self.settings).await;
                    } else {
                        warn!("Event::ProductCreated/Updated missing data");
                    }
                }
                Event::ProductUpdated(product_updated) => {
                    if let Some(product) = product_updated.clone().product {
                        product_updated_or_created(product_updated, &self.settings).await;
                    } else {
                        warn!("Event::ProductCreated/Updated missing data");
                    }
                }
                Event::ProductDeleted(product) => {
                    if let Some(product) = product.product {
                        delete(product.id.inner(), &self.settings).await;
                    } else {
                        warn!("Event::ProductDeleted missing data");
                    }
                }

                Event::CategoryCreated(category_created) => {
                    if let Some(category) = category_created.clone().category {
                        category_updated_or_created(category_created, category, &self.settings)
                            .await;
                    } else {
                        warn!("Event::CategoryCreated/Updated missing data");
                    }
                }
                Event::CategoryUpdated(category_updated) => {
                    if let Some(category) = category_updated.clone().category {
                        category_updated_or_created(category_updated, category, &self.settings)
                            .await;
                    } else {
                        warn!("Event::CategoryCreated/Updated missing data");
                    }
                }
                Event::CategoryDeleted(category) => {
                    if let Some(category) = category.category {
                        delete(category.id.inner(), &self.settings).await;
                    } else {
                        warn!("Event::CategoryDeleted missing data");
                    }
                }
                Event::ProductVariantCreated(variant) => {
                    if let Some(variant) = variant.product_variant {
                        product_updated_or_created(variant, &self.settings).await;
                    }
                }
                Event::ProductVariantUpdated(variant) => {
                    if let Some(variant) = variant.product_variant {
                        product_updated_or_created(variant, &self.settings).await;
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

fn product_updated_or_created<T: Serialize + Clone>(data: T, settings: &AppSettings) {}
