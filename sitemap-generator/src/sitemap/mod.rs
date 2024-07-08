mod category;
mod collection;
mod event_handler;
mod page;
mod product;

use chrono::{DateTime, FixedOffset, SubsecRound};
use quick_xml::DeError;
use serde::{Deserialize, Serialize};

const SITEMAP_XMLNS: &str = "http://sitemaps.org/schemas/sitemap/0.9";
const SALEOR_REF_XMLNS: &str = "http://app-sitemap-generator.kremik.sk/xml-schemas/saleor-ref.xsd";

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(rename = "urlset")]
pub struct UrlSet {
    #[serde(rename = "@xmlns:saleor")]
    xmlns_saleor: String,
    #[serde(rename = "@xmlns")]
    xmlns: String,
    pub url: Vec<Url>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct Url {
    pub loc: String,
    pub lastmod: DateTime<FixedOffset>,
    #[serde(rename = "saleor:ref")]
    pub saleor_ref: SaleorRef,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub enum RefType {
    Product,
    Category,
    Collection,
    Page,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct SaleorRef {
    #[serde(rename = "saleor:id")]
    pub id: String,
    #[serde(rename = "saleor:type")]
    pub typ: RefType,
    /**
    Related items come first in url, if present. eg:
        site.com/{page} : typ = RefType::Page
        site.com/{category}/{product} : typ= Product, related_typ: Category
    */
    #[serde(rename = "saleor:related-id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_id: Option<String>,
    /**
    Related items come first in url, if present. eg:
        site.com/{page} : typ = RefType::Page
        site.com/{category}/{product} : typ= Product, related_typ: Category
    */
    #[serde(rename = "saleor:related-typ")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_typ: Option<RefType>,
}

impl UrlSet {
    /**
    Icludes xml version header
    */
    pub fn to_file(&self) -> Result<String, DeError> {
        let init = quick_xml::se::to_string(self)?;
        Ok(r#"<?xml version="1.0" encoding="UTF-8"?>"#.to_string() + "\n" + &init)
    }
    /**
    adds static xmlns default strings
    */
    pub fn new() -> Self {
        let mut base_url = std::env::var("APP_API_BASE_URL").unwrap();
        //Cuz apparently xml url thingy isn't actually an url so you can't https? Gosh I hate xml
        if base_url.contains("https") {
            base_url = base_url.replacen("https", "http", 1);
        }
        //Trailing / in url would mess stuff up
        if base_url.chars().last().unwrap() == '/' {
            base_url.pop();
        }
        let xmlns_saleor = format!("{base_url}/schemas/saleor-ref.xsd",);
        Self {
            xmlns: SITEMAP_XMLNS.to_string(),
            xmlns_saleor,
            url: vec![],
        }
    }

    pub fn find_urls(&mut self, id: &str) -> Vec<&mut Url> {
        self.url
            .iter_mut()
            .filter(|url| {
                url.saleor_ref.id == id || url.saleor_ref.related_id == Some(id.to_owned())
            })
            .collect()
    }
}

impl Url {
    pub fn new(id: String, slug: String, typ: RefType) -> Self {
        Self {
            saleor_ref: SaleorRef {
                id,
                typ,
                related_id: None,
                related_typ: None,
            },
            lastmod: chrono::offset::Utc::now().fixed_offset().round_subsecs(1),
            // Have template string determine the url
            loc: format!("https://example.com/{slug}"),
        }
    }

    /**
    For exaple: product/category, product/collection
    */
    pub fn new_with_ref(
        id: String,
        slug: String,
        typ: RefType,
        related_id: Option<String>,
        related_slug: Option<String>,
        related_typ: Option<RefType>,
    ) -> Self {
        let loc = match related_slug {
            Some(r_s) => {
                format!("https://example.com/{r_s}/{slug}")
            }
            None => {
                format!("https://example.com/{slug}")
            }
        };
        Self {
            saleor_ref: SaleorRef {
                id,
                typ,
                related_id,
                related_typ,
            },
            lastmod: chrono::offset::Utc::now().fixed_offset().round_subsecs(1),
            // Have template string determine the url
            loc,
        }
    }
}
