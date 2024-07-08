use quick_xml::DeError;
use rayon::prelude::*;
use std::{
    fs::{self, read_dir, File},
    io::BufReader,
    path::PathBuf,
};
use tinytemplate::TinyTemplate;

use crate::{app::SitemapConfig, queries::event_subjects_updated::Event, sitemap::Url};
use tokio::{sync::mpsc::Receiver, task::JoinHandle};
use tracing::{debug, error, trace, warn};

use super::{RefType, UrlSet};

// 10k links google says, but there's also a size limit and my custom params might be messing with
// that? Rather split prematurely to be sure.
const MAX_URL_IN_SET: usize = 6000;

pub struct EventHandler {
    receiver: Receiver<(Event, SitemapConfig)>,
}

impl EventHandler {
    pub fn start(receiver: Receiver<(Event, SitemapConfig)>) -> JoinHandle<()> {
        let mut s = Self { receiver };
        tokio::spawn(s.listen())
    }

    async fn listen(mut self) {
        while let Some((message, sitemap_config)) = self.receiver.recv().await {
            match message {
                Event::ProductCreated(product) => {}
                Event::ProductUpdated(product) => {
                    if let Some(product) = product.product {
                        let mut url_sets = read_xmls(&sitemap_config.target_folder).await;
                        let mut was_any_set_affected = false;

                        //in case no sitemaps exist yet, create first urlset
                        if url_sets.is_empty() {
                            let url_set = UrlSet::new();
                            url_sets.push((
                                url_set,
                                std::path::Path::new(&format!(
                                    "{}/0.xml",
                                    sitemap_config.target_folder
                                ))
                                .to_path_buf(),
                            ));
                        }

                        // check if any url_sets contain affected urls
                        for (set, path) in &mut url_sets {
                            let mut affected_urls = set.find_urls(product.id.inner());

                            if affected_urls.len() == 0 {
                                trace!("Product doesn't exist in url_set {:?}", path);
                                continue;
                            }
                            was_any_set_affected = true;

                            // Update affected urls
                            affected_urls.iter_mut().for_each(|url| {
                                let mut templater = TinyTemplate::new();
                                templater
                                    .add_template("product", &sitemap_config.product_template)
                                    .expect("Check your url templates!");
                                let new_loc = templater
                                    .render("product", &product)
                                    .expect("Check your url templates!");
                                debug!("updated `{}` to `{}`", &url.loc, new_loc);
                                url.loc = new_loc;
                            });
                        }

                        //create product url if no set contained url with it
                        if !was_any_set_affected {
                            debug!("Product isn't in any sitemap, creating...");
                            if let Some((last_url_set, _)) = url_sets.last_mut() {
                                if product.category.is_none() {
                                    debug!("product missing category, hopefully not needed in url template?");
                                }
                                last_url_set.url.push(Url::new_with_ref(
                                    product.id.inner().to_owned(),
                                    product.slug,
                                    RefType::Product,
                                    product.category.clone().map(|c| c.id.inner().to_owned()),
                                    product.category.clone().map(|c| c.slug),
                                    Some(RefType::Category),
                                ));
                            }
                        }

                        let mut split_url_sets = vec![];
                        //write first time, if some throw too long error, split and try in second
                        //loop
                        for url_set in url_sets {
                            if let Err(e) = write_urlset_to_file(&url_set).await {
                                match e {
                                    WriteUrlSetToFileErr::UrlSetTooLong(l) => {
                                        debug!("url set too large ({l}), splitting...");
                                        if let Some(mut new_url_sets) =
                                            split_urlset_to_new_file(url_set).await
                                        {
                                            split_url_sets.append(&mut new_url_sets);
                                        }
                                    }
                                    e => error!("{:?}", e),
                                }
                            };
                        }

                        //the second attempt
                        for url_set in split_url_sets {
                            if let Err(e) = write_urlset_to_file(&url_set).await {
                                match e {
                                    WriteUrlSetToFileErr::UrlSetTooLong(l) => {
                                        error!("url set STILL too large?? ({l}), ignoring url set {:?}...", url_set);
                                    }
                                    e => error!("{:?}", e),
                                }
                            };
                        }
                    }
                    warn!("Event::ProductCreated missing product");
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

async fn read_xmls(target_folder: &str) -> Vec<(UrlSet, PathBuf)> {
    let paths = read_dir(target_folder).unwrap();
    let all_urls: Vec<(UrlSet, PathBuf)> = paths
        .into_iter()
        .par_bridge()
        .filter_map(|path| {
            if let Ok(path) = path {
                if path.path().is_file() {
                    let file = File::open(path.path()).expect("Unable to open file");
                    let reader = BufReader::new(file);
                    return Some((quick_xml::de::from_reader(reader).unwrap(), path.path()));
                }
            }
            return None;
        })
        .collect();
    all_urls
}

/**
* fails `if url_set.url.len() > MAX_URL_IN_SET`
*/
async fn split_urlset_to_new_file(union: (UrlSet, PathBuf)) -> Option<Vec<(UrlSet, PathBuf)>> {
    let (url_set, path) = union;

    if url_set.url.len() < MAX_URL_IN_SET {
        return None;
    }

    let mut was_original_file_assigned = false;
    let chunks = url_set.url.chunks(MAX_URL_IN_SET).collect::<Vec<_>>();

    let mut file_number = path
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .parse::<i32>()
        .unwrap();

    return Some(
        chunks
            .into_iter()
            .map(|urls| {
                let folder = path.clone().parent().unwrap().to_str().unwrap().to_owned();

                //keep incrementing file number till a file with that number is free to use
                if !was_original_file_assigned {
                    was_original_file_assigned = true
                } else {
                    while !std::path::Path::new(&format!("{folder}/{file_number}.xml")).exists() {
                        file_number = file_number + 1;
                    }
                }

                let mut url_set = UrlSet::new();
                url_set.url = urls.into();
                (
                    url_set,
                    std::path::Path::new(&format!("{folder}/{file_number}.xml")).to_path_buf(),
                )
            })
            .collect::<Vec<_>>(),
    );
}

async fn write_urlset_to_file(
    url_set_n_path: &(UrlSet, PathBuf),
) -> Result<(), WriteUrlSetToFileErr> {
    let (url_set, path) = url_set_n_path;
    if url_set.url.len() > MAX_URL_IN_SET {
        return Err(WriteUrlSetToFileErr::UrlSetTooLong(url_set.url.len()));
    }
    fs::write(path, &quick_xml::se::to_string(&url_set)?)?;
    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum WriteUrlSetToFileErr {
    #[error("writing error")]
    IoResult(#[from] std::io::Error),
    #[error("Url set length exeeds xml standard of 10k entries per file")]
    UrlSetTooLong(usize),
    #[error("{0}")]
    DeError(#[from] DeError),
}
