use std::{
    fs::{self, read_dir, File},
    io::{BufReader, ErrorKind},
    path::PathBuf,
};
use tinytemplate::TinyTemplate;

use crate::{
    app::SitemapConfig,
    queries::event_subjects_updated::{Event, Product},
    sitemap::{ItemType, Url},
};
use tokio::{sync::mpsc::Receiver, task::JoinHandle};
use tracing::{debug, error, trace, warn};

use super::UrlSet;

// 10k links google says, but there's also a size limit and my custom params might be messing with
// that? Rather split prematurely to be sure.
const MAX_URL_IN_SET: usize = 6000;
const DB_FILE_NAME: &str = "db.toml";

pub struct EventHandler {
    receiver: Receiver<(Event, SitemapConfig)>,
}

impl EventHandler {
    pub fn start(receiver: Receiver<(Event, SitemapConfig)>) -> JoinHandle<()> {
        let s = Self { receiver };
        tokio::spawn(s.listen())
    }

    async fn listen(mut self) {
        while let Some((message, sitemap_config)) = self.receiver.recv().await {
            match message {
                Event::ProductCreated(product) => {
                    if let Some(product) = product.product {
                        product_update_or_create(product, sitemap_config).await;
                    }
                    warn!("Event::ProductCreated missing product");
                }
                Event::ProductUpdated(product) => {
                    if let Some(product) = product.product {
                        product_update_or_create(product, sitemap_config).await;
                    }
                    warn!("Event::ProductUpdated missing product");
                }
                Event::ProductDeleted(product) => {}
                Event::CategoryCreated(category) => {}
                Event::CategoryUpdated(category) => {}
                Event::CategoryDeleted(category) => {}
                Event::CollectionCreated(collection) => {}
                Event::CollectionUpdated(collection) => {}
                Event::CollectionDeleted(collection) => {}
                Event::PageCreated(page) => {}
                Event::PageUpdated(page) => {}
                Event::PageDeleted(page) => {}
                Event::Unknown => warn!("Unknown event called"),
            }
        }
    }
}

async fn product_delete(product: Product, sitemap_config: SitemapConfig) {
    let mut url_set = match get_from_file(&sitemap_config.target_folder).await {
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

    url_set.flush_related(product.id.inner());

    write_to_file(&url_set, &sitemap_config.target_folder)
        .await
        .unwrap();
}

async fn product_update_or_create(product: Product, sitemap_config: SitemapConfig) {
    let mut url_set = match get_from_file(&sitemap_config.target_folder).await {
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

    let mut affected_urls = url_set.find_affected(product.id.inner(), &product.slug);
    debug!("affected urls: {:?}", &affected_urls);

    if affected_urls.len() == 0 {
        trace!("Product doesn't exist in url_set yet");
        url_set.push(Url::new_product(&sitemap_config.product_template, product).unwrap());
    } else {
        // Update affected urls
        affected_urls.iter_mut().for_each(|url| {
            let mut templater = TinyTemplate::new();
            templater
                .add_template("product", &sitemap_config.product_template)
                .expect("Check your url templates!");
            let new_loc = templater
                .render("product", &product)
                .expect("Check your url templates!");
            debug!("updated `{}` to `{}`", &url.url, new_loc);
            url.url = new_loc;
        });
    }
    write_to_file(&url_set, &sitemap_config.target_folder)
        .await
        .unwrap();
}

async fn get_from_file(target_folder: &str) -> Result<UrlSet, UrlSetFileOperationsErr> {
    let urls: UrlSet =
        serde_cbor::de::from_slice(&std::fs::read(format!("{target_folder}/{DB_FILE_NAME}"))?)?;
    Ok(urls)
}

async fn write_to_file(
    url_set: &UrlSet,
    target_folder: &str,
) -> Result<(), UrlSetFileOperationsErr> {
    if url_set.len() > MAX_URL_IN_SET {
        // return Err(UrlSetFileOperationsErr::UrlSetTooLong(url_set.len()));
        warn!("Urlset exeeded {MAX_URL_IN_SET} links, search engines might start to complain!");
    }
    fs::write(
        format!("{target_folder}/{DB_FILE_NAME}"),
        &serde_cbor::to_vec(url_set)?,
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
