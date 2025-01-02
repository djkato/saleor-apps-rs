pub mod event_handler;
pub mod regenerate;

use std::ops::{Deref, DerefMut};

use saleor_app_sdk::webhooks::{utils::EitherWebhookType, AsyncWebhookEventType};
use serde::{Deserialize, Serialize};
use tinytemplate::TinyTemplate;
use tracing::debug;

use crate::{
    app::SitemapConfig,
    queries::event_subjects_updated::{
        Category, Category2, CategoryCreated, CategoryDeleted, Collection, CollectionCreated,
        CollectionDeleted, Page, PageCreated, PageDeleted, Product, ProductCreated, ProductDeleted,
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

    pub fn flush_related(&mut self, id: &str) {
        self.retain(|u| u.data.id != id && u.related.as_ref().is_some_and(|ud| ud.id != id));
    }

    pub fn find_related(&mut self, id: &str) -> Vec<&mut Url> {
        self.iter_mut()
            .filter(|u| u.data.id == id || u.related.as_ref().is_some_and(|ud| ud.id == id))
            .collect()
    }

    pub fn find_affected(&mut self, id: &str, slug: &str) -> AffectedResult<'_> {
        let related: Vec<&mut Url> = self.find_related(id);
        debug!("related urls: {:?}", &related);
        if related.is_empty() {
            return AffectedResult::NoneRelated;
        }

        let affected = related
            .into_iter()
            .filter(|u| {
                (u.data.id == id && u.data.slug != slug)
                    || u.related
                        .as_ref()
                        .is_some_and(|r| (r.id == id && r.slug != slug))
            })
            .map(|u| match u.data.id == id {
                true => AffectedType::Data(u),
                false => AffectedType::RelatedData(u),
            })
            .collect::<Vec<_>>();
        if affected.is_empty() {
            return AffectedResult::NoneAffected;
        }

        AffectedResult::Some(affected)
    }
}

#[derive(Debug)]
pub enum AffectedResult<'a> {
    Some(Vec<AffectedType<&'a mut Url>>),
    NoneAffected,
    NoneRelated,
}

#[derive(Debug)]
pub enum AffectedType<T> {
    Data(T),
    RelatedData(T),
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
    pub fn into_event_updated_body(self, slug_postfix: &str) -> (String, EitherWebhookType) {
        match self.data.typ.clone() {
            ItemType::Product => {
                let mut data: ProductCreated = self.into();
                data.product = data.product.map(|mut p| {
                    p.slug = p.slug.clone() + slug_postfix;
                    p
                });
                // debug!("{:?}", &data);
                (
                    serde_json::to_string_pretty(&data).unwrap(),
                    EitherWebhookType::Async(AsyncWebhookEventType::ProductUpdated),
                )
            }
            ItemType::Category => {
                let mut data: CategoryCreated = self.into();
                data.category = data.category.map(|mut p| {
                    p.slug = p.slug.clone() + slug_postfix;
                    p
                });
                (
                    serde_json::to_string_pretty(&data).unwrap(),
                    EitherWebhookType::Async(AsyncWebhookEventType::CategoryUpdated),
                )
            }
            ItemType::Page => {
                let mut data: PageCreated = self.into();
                data.page = data.page.map(|mut p| {
                    p.slug = p.slug.clone() + slug_postfix;
                    p
                });
                (
                    serde_json::to_string_pretty(&data).unwrap(),
                    EitherWebhookType::Async(AsyncWebhookEventType::PageUpdated),
                )
            }

            ItemType::Collection => {
                let mut data: CollectionCreated = self.into();
                data.collection = data.collection.map(|mut p| {
                    p.slug = p.slug.clone() + slug_postfix;
                    p
                });
                (
                    serde_json::to_string_pretty(&data).unwrap(),
                    EitherWebhookType::Async(AsyncWebhookEventType::CollectionUpdated),
                )
            }
        }
    }
}

impl From<Url> for ProductCreated {
    fn from(value: Url) -> Self {
        Self {
            product: Some(Product {
                slug: value.data.slug,
                id: cynic::Id::new(value.data.id),
                category: value.related.map(|c| Category {
                    slug: c.slug,
                    id: cynic::Id::new(c.id),
                }),
            }),
        }
    }
}

impl From<Url> for CategoryCreated {
    fn from(value: Url) -> Self {
        Self {
            category: Some(Category2 {
                slug: value.data.slug,
                id: cynic::Id::new(value.data.id),
            }),
        }
    }
}

impl From<Url> for CollectionCreated {
    fn from(value: Url) -> Self {
        Self {
            collection: Some(Collection {
                slug: value.data.slug,
                id: cynic::Id::new(value.data.id),
            }),
        }
    }
}

impl From<Url> for PageCreated {
    fn from(value: Url) -> Self {
        Self {
            page: Some(Page {
                slug: value.data.slug,
                id: cynic::Id::new(value.data.id),
            }),
        }
    }
}

impl From<Url> for ProductDeleted {
    fn from(value: Url) -> Self {
        Self {
            product: Some(Product {
                slug: value.data.slug,
                id: cynic::Id::new(value.data.id),
                category: value.related.map(|c| Category {
                    slug: c.slug,
                    id: cynic::Id::new(c.id),
                }),
            }),
        }
    }
}

impl From<Url> for CategoryDeleted {
    fn from(value: Url) -> Self {
        Self {
            category: Some(Category2 {
                slug: value.data.slug,
                id: cynic::Id::new(value.data.id),
            }),
        }
    }
}

impl From<Url> for CollectionDeleted {
    fn from(value: Url) -> Self {
        Self {
            collection: Some(Collection {
                slug: value.data.slug,
                id: cynic::Id::new(value.data.id),
            }),
        }
    }
}

impl From<Url> for PageDeleted {
    fn from(value: Url) -> Self {
        Self {
            page: Some(Page {
                slug: value.data.slug,
                id: cynic::Id::new(value.data.id),
            }),
        }
    }
}
#[derive(thiserror::Error, Debug)]
pub enum NewUrlError {
    #[error("Some property inside passed data for new url was None, but should've been Some")]
    MissingData,
    #[error("Bad templates or wrong context data to fill out the template")]
    BadTemplating(#[from] tinytemplate::error::Error),
}
