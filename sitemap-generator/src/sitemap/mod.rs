pub mod event_handler;
pub mod regenerate;

use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};
use tinytemplate::TinyTemplate;
use tracing::debug;

use crate::app::SitemapConfig;

const SITEMAP_XMLNS: &str = "http://sitemaps.org/schemas/sitemap/0.9";
const SALEOR_REF_XMLNS: &str = "http://app-sitemap-generator.kremik.sk/xml-schemas/saleor-ref.xsd";

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(rename = "urlset")]
pub struct UrlSet {
    pub urls: Vec<Url>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct Url {
    pub url: String,
    pub data: ItemData,
    pub related: Option<ItemData>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct ItemData {
    pub id: String,
    pub slug: String,
    pub typ: ItemType,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub enum ItemType {
    Product,
    Category,
    Collection,
    Page,
}

impl UrlSet {
    pub fn new() -> Self {
        Self { urls: vec![] }
    }

    pub fn flush_related(&mut self, id: &str) {
        self.retain(|u| u.data.id != id && u.related.as_ref().map_or(true, |ud| ud.id != id));
    }

    pub fn find_related(&mut self, id: &str) -> Vec<&mut Url> {
        self.iter_mut()
            .filter(|u| u.data.id == id || u.related.as_ref().map_or(false, |ud| ud.id == id))
            .collect()
    }

    pub fn find_affected(&mut self, id: &str, slug: &str) -> Vec<&mut Url> {
        self.iter_mut()
            .filter(|u| {
                debug!(
                    "comparing: ( {} == {} && {} != {} ) || ( {:?} == {} && {:?} != {} )",
                    &u.data.id,
                    &id,
                    &u.data.slug,
                    &slug,
                    u.related.clone().map(|ud| ud.id),
                    &id,
                    u.related.clone().map(|ud| ud.slug),
                    &slug
                );
                (u.data.id == id && u.data.slug != slug)
                    || (u
                        .related
                        .as_ref()
                        .map_or(false, |ud| ud.id == id && ud.slug != slug))
            })
            .collect()
    }
}

impl Deref for UrlSet {
    type Target = Vec<Url>;
    fn deref(&self) -> &Self::Target {
        &self.urls
    }
}

impl DerefMut for UrlSet {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.urls
    }
}

impl Url {
    pub fn new<T: Serialize>(
        data: T,
        sitemap_config: &SitemapConfig,
        item: ItemData,
        rel_item: Option<ItemData>,
    ) -> Result<Self, NewUrlError> {
        let mut tt = TinyTemplate::new();

        tt.add_template(
            "t",
            match item.typ {
                ItemType::Category => &sitemap_config.category_template,
                ItemType::Page => &sitemap_config.pages_template,
                ItemType::Collection => &sitemap_config.collection_template,
                ItemType::Product => &sitemap_config.product_template,
            },
        )?;
        let url = tt.render("t", &data)?;
        Ok(Self {
            url,
            data: item,
            related: rel_item,
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum NewUrlError {
    #[error("Some property inside passed data for new url was None, but should've been Some")]
    MissingData,
    #[error("Bad templates or wrong context data to fill out the template")]
    BadTemplating(#[from] tinytemplate::error::Error),
}
