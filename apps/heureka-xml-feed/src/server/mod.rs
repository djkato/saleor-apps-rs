use std::{
    ops::{Deref, DerefMut},
    str::FromStr,
};

use graphqls::get_category_parents;
use heureka_xml_feed::{Delivery, Shop, ShopItem};
use rust_decimal::{Decimal, prelude::FromPrimitive};
use serde::{Serialize, de::DeserializeOwned};
use surrealdb::{Surreal, engine::any::Any};
use tinytemplate::TinyTemplate;
use url::Url;

use crate::{
    app::AppSettings,
    queries::{
        products_variants_categories::{Category, Product, ProductVariant, ProductVariant2},
        surreal_types,
    },
};

pub mod event_handler;
pub mod graphqls;
pub mod surrealdbs;

pub fn try_create_shopitem(
    product: surreal_types::Product,
    variant: surreal_types::ProductVariant,
    deliveries: Vec<Delivery>,
    heureka_categorytext: String,
    variant_url: Url,
    tax_rate: String,
) -> Result<ShopItem, TryIntoShopItemError> {
    let mut description = None;
    if let Some(d) = product.description {
        description = Some(d);
    }

    //I need firmt imgurl to be single, then rest in imgurl_alternative
    let media = variant
        .media
        .clone()
        .into_iter()
        .map(|u| u.url)
        .collect::<Vec<_>>();
    let media = media
        .split_first()
        .ok_or(TryIntoShopItemError::MissingMedia)?;

    Ok(ShopItem {
        item_id: variant.id.into_inner(),
        url: Some(variant_url.clone()),
        productname: variant.name,
        price_vat: variant
            .pricing
            .and_then(|p| {
                p.price
                    .and_then(|p| Decimal::from_f32(p.gross.amount as f32))
            })
            .ok_or(TryIntoShopItemError::PricingMissingOrFailed)?,
        vat: Some(tax_rate),
        imgurl: Url::from_str(media.0)?,
        imgurl_alternative: media
            .1
            .into_iter()
            .map(|m| Url::from_str(m))
            .collect::<Result<Vec<_>, _>>()?,
        delivery: deliveries.clone(),
        productno: variant.sku,
        description,
        categorytext: heureka_categorytext.clone(),
        /*get_category_text_from_product(product.clone())
        .unwrap_or("".to_owned()),*/
        accessory: vec![],
        delivery_date: None,
        dues: None,
        ean: None,
        extended_warranty: None,
        gift: None,
        gift_id: None,
        heureka_cpc: None,
        isbn: None,
        item_type: None,
        itemgroup_id: None,
        manufacturer: None,
        product: None,
        special_service: vec![],
        video_url: None,
        param: vec![],
    })
}

#[derive(thiserror::Error, Debug)]
pub enum TryIntoShopError {
    #[error("Failed surrealdb query: {0}")]
    SurrealDBError(#[from] surrealdb::Error),
    #[error("Failed converting item to shopitem, {0}")]
    ShopItemError(#[from] TryIntoShopItemError),
}

#[derive(thiserror::Error, Debug)]
pub enum TryIntoShopItemError {
    #[error("Product is missing variants")]
    MissingVariants,
    #[error("Failed creating variant url from template, {0}")]
    UrlFromTeplateError(#[from] TryUrlFromTemplateError),
    #[error("Failed creating price from variant")]
    PricingMissingOrFailed,
    #[error("Missing media/images for variant")]
    MissingMedia,
    #[error("Failed converting media url from string, {0}")]
    UrlFromStrError(#[from] url::ParseError),
    #[error("failed converting description from json to string, {0}")]
    SerdeJson(#[from] serde_json::Error),
}

pub fn variant_url_from_template<'a, T: Serialize>(
    url_template: String,
    context: &'a T,
) -> Result<Url, TryUrlFromTemplateError> {
    let mut tt = TinyTemplate::new();

    tt.add_template("t", &url_template)?;

    Ok(url::Url::from_str(&tt.render("t", context)?)?)
}

#[derive(Debug, Clone, Serialize)]
pub struct VariantUrlTemplateContext<'a> {
    product: &'a surreal_types::Product,
    variant: &'a surreal_types::ProductVariant,
    category: &'a surreal_types::Category,
}

#[derive(thiserror::Error, Debug)]
pub enum TryUrlFromTemplateError {
    #[error("Something errorred during templating, {0}")]
    TinyTemplateError(#[from] tinytemplate::error::Error),
    #[error("failed parsing url created from urltemplate, {0}")]
    UrlParseError(#[from] url::ParseError),
}

pub fn find_category_text(categories: &Vec<surreal_types::Category>) -> Option<String> {
    for c in categories {
        if let Some(m) = c.metafield.clone() {
            return Some(m);
        }
    }
    None
}
