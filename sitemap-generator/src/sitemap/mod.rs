mod category;
mod collection;
mod event_handler;
mod page;
mod product;

use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};
use tinytemplate::TinyTemplate;

use crate::{
    app::SitemapConfig,
    queries::{
        event_subjects_updated::{Category, Collection, Page, Product, ProductUpdated},
        get_all_categories_n_products::Product,
    },
};

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
    pub fn new_product(
        sitemap_config: &SitemapConfig,
        product: Product,
    ) -> Result<Self, NewUrlError> {
        let category = product
            .category
            .as_ref()
            .ok_or(NewUrlError::MissingData)?
            .clone();
        let data = ItemData {
            id: product.id.inner().to_owned(),
            slug: product.slug.clone(),
            typ: ItemType::Product,
        };

        let related = Some(ItemData {
            id: category.id.inner().to_owned(),
            slug: category.slug,
            typ: ItemType::Category,
        });

        let mut tt = TinyTemplate::new();

        tt.add_template("t", &sitemap_config.product_template);

        let url = tt.render("t", &product)?;
        Ok(Self { url, data, related })
    }

    pub fn new_category(
        sitemap_config: &SitemapConfig,
        category: Category,
    ) -> Result<Self, NewUrlError> {
        let data = ItemData {
            id: category.id.inner().to_owned(),
            slug: category.slug.clone(),
            typ: ItemType::Category,
        };
        let mut tt = TinyTemplate::new();

        tt.add_template("t", &sitemap_config.category_template);

        let url = tt.render("t", &category)?;
        Ok(Self {
            url,
            data,
            related: None,
        })
    }

    pub fn new_collection(
        sitemap_config: &SitemapConfig,
        collection: Collection,
    ) -> Result<Self, NewUrlError> {
        let data = ItemData {
            id: collection.id.inner().to_owned(),
            slug: collection.slug.clone(),
            typ: ItemType::Collection,
        };
        let mut tt = TinyTemplate::new();

        tt.add_template("t", &sitemap_config.collection_template);

        let url = tt.render("t", &collection)?;
        Ok(Self {
            url,
            data,
            related: None,
        })
    }

    pub fn new_page(sitemap_config: &SitemapConfig, page: Page) -> Result<Self, NewUrlError> {
        let data = ItemData {
            id: page.id.inner().to_owned(),
            slug: page.slug.clone(),
            typ: ItemType::Page,
        };
        let mut tt = TinyTemplate::new();

        tt.add_template("t", &sitemap_config.pages_template);

        let url = tt.render("t", &page)?;
        Ok(Self {
            url,
            data,
            related: None,
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
