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



#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(rename = "urlset")]
pub struct UrlSet {
    #[serde(rename = "@xmlns:saleor")]
    xmlns_saleor: String,
    #[serde(rename = "@xmlns")]
    xmlns: String,
    pub url: Vec<Url>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Url {
    pub loc: String,
    pub lastmod: DateTime<FixedOffset>,
    #[serde(rename = "saleor:ref")]
    pub saleor_ref: SaleorRef,
}
pub enum RefType {
    Product,
    
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct SaleorRef {
    #[serde(rename = "saleor:id")]
    pub id: String,
    #[serde(rename = "saleor:type")]
    pub typ: String,
    #[serde(rename = "saleor:category-id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<String>,
    pub 
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
}

impl Url {
    pub fn new_generic_url(id: String, slug: String) -> Self {
        Self {
            saleor_ref: SaleorRef {
                product_id: None,
                category_id: Some(id),
            },
            lastmod: chrono::offset::Utc::now().fixed_offset().round_subsecs(1),
            // Have template string determine the url
            loc: format!("https://example.com/{slug}"),
        }
    }

    pub fn new_product_url(
        category_id: String,
        product_id: String,
        category_slug: String,
        product_slug: String,
    ) -> Self {
        Self {
            // Have template string determine the url
            loc: format!("https://example.com/{category_slug}/{product_slug}"),
            lastmod: chrono::offset::Utc::now().fixed_offset().round_subsecs(1),
            saleor_ref: SaleorRef {
                product_id: Some(product_id),
                category_id: Some(category_id),
            },
        }
    }
}
