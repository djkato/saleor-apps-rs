use super::regenerate::regenerate;
use serde::Serialize;
use std::{
    fs::{self},
    io::ErrorKind,
};
use tinytemplate::TinyTemplate;

use crate::{
    app::{AppState, SitemapConfig},
    queries::event_subjects_updated::{
        Category2, CategoryCreated, CategoryDeleted, CategoryUpdated, Collection,
        CollectionCreated, CollectionDeleted, CollectionUpdated, Page, PageCreated, PageDeleted,
        PageUpdated, Product, ProductCreated, ProductDeleted, ProductUpdated,
    },
    sitemap::Url,
};
use tokio::{sync::mpsc::Receiver, task::JoinHandle};
use tracing::{debug, error, info, trace, warn};

use super::{ItemData, ItemType, UrlSet};

// 10k links google says, but there's also a size limit and my custom params might be messing with
// that? Rather split prematurely to be sure.
const MAX_URL_IN_SET: usize = 50_000;
const DB_FILE_NAME: &str = "db.toml";
const SITEMAP_FILE_NAME: &str = "sitemap.txt";

pub struct EventHandler {
    receiver: Receiver<Event>,
    sitemap_config: SitemapConfig,
}

#[derive(Debug, Clone)]
pub enum Event {
    ProductUpdated(ProductUpdated),
    ProductCreated(ProductCreated),
    ProductDeleted(ProductDeleted),
    CategoryCreated(CategoryCreated),
    CategoryUpdated(CategoryUpdated),
    CategoryDeleted(CategoryDeleted),
    PageCreated(PageCreated),
    PageUpdated(PageUpdated),
    PageDeleted(PageDeleted),
    CollectionCreated(CollectionCreated),
    CollectionUpdated(CollectionUpdated),
    CollectionDeleted(CollectionDeleted),
    Regenerate(RegenerateEvent),
    Unknown,
}

#[derive(Debug, Clone)]
pub struct RegenerateEvent {
    pub state: AppState,
    pub saleor_api_url: String,
}

impl EventHandler {
    pub fn start(sitemap_config: SitemapConfig, receiver: Receiver<Event>) -> JoinHandle<()> {
        let s = Self {
            sitemap_config,
            receiver,
        };
        tokio::spawn(s.listen())
    }

    async fn listen(mut self) {
        while let Some(message) = self.receiver.recv().await {
            debug!("received Event: {:?}", &message);
            match message {
                Event::ProductCreated(product_created) => {
                    if let Some(product) = product_created.clone().product {
                        product_updated_or_created(product_created, product, &self.sitemap_config)
                            .await;
                    } else {
                        warn!("Event::ProductCreated/Updated missing data");
                    }
                }
                Event::ProductUpdated(product_updated) => {
                    if let Some(product) = product_updated.clone().product {
                        product_updated_or_created(product_updated, product, &self.sitemap_config)
                            .await;
                    } else {
                        warn!("Event::ProductCreated/Updated missing data");
                    }
                }
                Event::ProductDeleted(product) => {
                    if let Some(product) = product.product {
                        delete(product.id.inner(), &self.sitemap_config).await;
                    } else {
                        warn!("Event::ProductDeleted missing data");
                    }
                }

                Event::CategoryCreated(category_created) => {
                    if let Some(category) = category_created.clone().category {
                        category_updated_or_created(
                            category_created,
                            category,
                            &self.sitemap_config,
                        )
                        .await;
                    } else {
                        warn!("Event::CategoryCreated/Updated missing data");
                    }
                }
                Event::CategoryUpdated(category_updated) => {
                    if let Some(category) = category_updated.clone().category {
                        category_updated_or_created(
                            category_updated,
                            category,
                            &self.sitemap_config,
                        )
                        .await;
                    } else {
                        warn!("Event::CategoryCreated/Updated missing data");
                    }
                }
                Event::CategoryDeleted(category) => {
                    if let Some(category) = category.category {
                        delete(category.id.inner(), &self.sitemap_config).await;
                    } else {
                        warn!("Event::CategoryDeleted missing data");
                    }
                }

                Event::CollectionCreated(collection_created) => {
                    if let Some(collection) = collection_created.clone().collection {
                        collection_updated_or_created(
                            collection_created,
                            collection,
                            &self.sitemap_config,
                        )
                        .await;
                    } else {
                        warn!("Event::ProductCreated/Updated missing Data");
                    }
                }
                Event::CollectionUpdated(collection_updated) => {
                    if let Some(collection) = collection_updated.clone().collection {
                        collection_updated_or_created(
                            collection_updated,
                            collection,
                            &self.sitemap_config,
                        )
                        .await;
                    } else {
                        warn!("Event::ProductCreated/Updated missing Data");
                    }
                }
                Event::CollectionDeleted(collection) => {
                    if let Some(collection) = collection.collection {
                        delete(collection.id.inner(), &self.sitemap_config).await;
                    } else {
                        warn!("Event::ProductDeleted missing data");
                    }
                }

                Event::PageCreated(page_created) => {
                    if let Some(page) = page_created.clone().page {
                        page_updated_or_created(page_created, page, &self.sitemap_config).await;
                    }
                    warn!("Event::PageCreated/Updated missing data");
                }
                Event::PageUpdated(page_updated) => {
                    if let Some(page) = page_updated.clone().page {
                        page_updated_or_created(page_updated, page, &self.sitemap_config).await;
                    } else {
                        warn!("Event::PageCreated/Updated missing data");
                    }
                }
                Event::PageDeleted(page) => {
                    if let Some(page) = page.page {
                        delete(page.id.inner(), &self.sitemap_config).await;
                    } else {
                        warn!("Event::PageDeleted missing data");
                    }
                }
                Event::Regenerate(r) => match regenerate(r.state, r.saleor_api_url).await {
                    Ok(_) => info!("regenerate: Fully created sitemap!"),
                    Err(e) => error!("regenerate: ERR! {:?}", e),
                },
                Event::Unknown => (),
            }
            debug!("Event succesfully handled");
        }
    }
}

/* =============== Event handlers =============== */

async fn product_updated_or_created<T: Serialize>(
    request: T,
    product: Product,
    sitemap_config: &SitemapConfig,
) {
    update_or_create(
        request,
        sitemap_config,
        ItemData {
            id: product.id.inner().to_owned(),
            slug: product.slug,
            typ: ItemType::Product,
        },
        product.category.map(|c| ItemData {
            slug: c.slug,
            typ: ItemType::Category,
            id: c.id.inner().to_owned(),
        }),
    )
    .await;
}

async fn category_updated_or_created<T: Serialize>(
    request: T,
    category: Category2,
    sitemap_config: &SitemapConfig,
) {
    update_or_create(
        request,
        sitemap_config,
        ItemData {
            id: category.id.inner().to_owned(),
            slug: category.slug,
            typ: ItemType::Category,
        },
        None,
    )
    .await;
}

async fn page_updated_or_created<T: Serialize>(
    request: T,
    page: Page,
    sitemap_config: &SitemapConfig,
) {
    update_or_create(
        request,
        sitemap_config,
        ItemData {
            id: page.id.inner().to_owned(),
            slug: page.slug,
            typ: ItemType::Page,
        },
        None,
    )
    .await;
}

async fn collection_updated_or_created<T: Serialize>(
    request: T,
    collection: Collection,
    sitemap_config: &SitemapConfig,
) {
    update_or_create(
        request,
        sitemap_config,
        ItemData {
            id: collection.id.inner().to_owned(),
            slug: collection.slug,
            typ: ItemType::Collection,
        },
        None,
    )
    .await;
}

/* ============= URL Manipulations ================ */

async fn update_or_create<T: Serialize>(
    data: T,
    sitemap_config: &SitemapConfig,
    item: ItemData,
    rel_item: Option<ItemData>,
) {
    let mut url_set = match get_db_from_file(&sitemap_config.target_folder).await {
        Ok(u) => u,
        Err(e) => match e {
            UrlSetFileOperationsErr::IoResult(e) => match e.kind() {
                ErrorKind::NotFound => UrlSet::new(),
                _ => {
                    error!("File errror: {:?}\n won't crash, but probably broken.", e);
                    return;
                }
            },
            UrlSetFileOperationsErr::DeError(e) => {
                error!(
                    "DE error: {:?}\n Won't crash, but something went badly wrong",
                    e
                );
                return;
            }
        },
    };

    let mut affected_urls = url_set.find_affected(&item.id, &item.slug);
    debug!("affected urls: {:?}", &affected_urls);

    if affected_urls.is_empty() {
        trace!("{:?} doesn't exist in url_set yet", &item.slug);
        url_set.push(Url::new(data, sitemap_config, item, rel_item).unwrap());
    } else {
        // Update affected urls
        affected_urls.iter_mut().for_each(|url| {
            let mut templater = TinyTemplate::new();
            templater
                .add_template("product", &sitemap_config.product_template)
                .expect("Check your url templates!");
            let new_loc = templater
                .render("product", &data)
                .expect("Check your url templates!");
            debug!("updated `{}` to `{}`", &url.url, new_loc);
            url.url = new_loc;
        });
    }
    write_db_to_file(&url_set, &sitemap_config.target_folder)
        .await
        .unwrap();
    write_url_set_to_file(&url_set, &sitemap_config.target_folder)
        .await
        .unwrap();
}

async fn delete(id: &str, sitemap_config: &SitemapConfig) {
    let mut url_set = match get_db_from_file(&sitemap_config.target_folder).await {
        Ok(u) => u,
        Err(e) => match e {
            UrlSetFileOperationsErr::IoResult(e) => match e.kind() {
                ErrorKind::NotFound => UrlSet::new(),
                _ => {
                    error!("File errror: {:?}\n won't crash, but probably broken.", e);
                    return;
                }
            },
            UrlSetFileOperationsErr::DeError(e) => {
                error!(
                    "DE error: {:?}\n Won't crash, but something went badly wrong",
                    e
                );
                return;
            }
        },
    };
    url_set.flush_related(id);

    write_db_to_file(&url_set, &sitemap_config.target_folder)
        .await
        .unwrap();
    write_url_set_to_file(&url_set, &sitemap_config.target_folder)
        .await
        .unwrap();
}

/* =================== File and SerDe operations  ========================= */

async fn get_db_from_file(target_folder: &str) -> Result<UrlSet, UrlSetFileOperationsErr> {
    let urls: UrlSet =
        serde_json::de::from_slice(&std::fs::read(format!("{target_folder}/{DB_FILE_NAME}"))?)
            .unwrap();
    Ok(urls)
}

pub async fn write_db_to_file(
    url_set: &UrlSet,
    target_folder: &str,
) -> Result<(), UrlSetFileOperationsErr> {
    if url_set.len() > MAX_URL_IN_SET {
        // return Err(UrlSetFileOperationsErr::UrlSetTooLong(url_set.len()));
        warn!("Urlset exeeded {MAX_URL_IN_SET} links, search engines might start to complain!");
    }
    fs::write(
        format!("{target_folder}/{DB_FILE_NAME}"),
        serde_json::to_vec(url_set).unwrap(),
    )?;
    Ok(())
}

pub async fn write_url_set_to_file(
    url_set: &UrlSet,
    target_folder: &str,
) -> Result<(), UrlSetFileOperationsErr> {
    if url_set.len() > MAX_URL_IN_SET {
        // return Err(UrlSetFileOperationsErr::UrlSetTooLong(url_set.len()));
        warn!("Urlset exeeded {MAX_URL_IN_SET} links, search engines might start to complain!");
    }

    fs::write(
        format!("{target_folder}/{SITEMAP_FILE_NAME}"),
        url_set
            .iter()
            .map(|u| u.url.clone())
            .collect::<Vec<_>>()
            .join("\n"),
    )?;
    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum UrlSetFileOperationsErr {
    #[error("writing error")]
    IoResult(#[from] std::io::Error),
    // #[error("Url set length exeeds xml standard of 10k entries per file")]
    // UrlSetTooLong(usize),
    #[error("{0}")]
    DeError(#[from] serde_cbor::Error),
}
