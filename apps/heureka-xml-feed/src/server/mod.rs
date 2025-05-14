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
    queries::products_variants_categories::{Category, Product, ProductVariant, ProductVariant2},
};

pub mod event_handler;
pub mod graphqls;
pub mod surrealdbs;

pub fn try_shopitem_from_product<'a, T: Serialize>(
    product: Product,
    variants: Vec<ProductVariant>,
    deliveries: Vec<Delivery>,
    heureka_categorytext: String,
    variant_url: Url,
    settings: AppSettings,
) -> Result<Vec<ShopItem>, TryIntoShopItemError> {
    match product.clone().variants {
        None => Err(TryIntoShopItemError::MissingVariants),
        Some(variants) => {
            let mut shopitems: Vec<ShopItem> = vec![];
            for v in variants.into_iter() {
                //I need firmt imgurl to be single, then rest in imgurl_alternative
                let media = v
                    .media
                    .clone()
                    .and_then(|m| Some(m.into_iter().map(|u| u.url).collect::<Vec<_>>()))
                    .ok_or(TryIntoShopItemError::MissingMedia)?;
                let media = media
                    .split_first()
                    .ok_or(TryIntoShopItemError::MissingMedia)?;

                shopitems.push(ShopItem {
                    item_id: v.id.into_inner(),
                    url: Some(variant_url.clone()),
                    productname: v.name,
                    price_vat: v
                        .pricing
                        .and_then(|p| {
                            p.price
                                .and_then(|p| Decimal::from_f32(p.gross.amount as f32))
                        })
                        .ok_or(TryIntoShopItemError::PricingMissingOrFailed)?,
                    vat: Some(settings.clone().tax_rate),
                    imgurl: Url::from_str(media.0)?,
                    imgurl_alternative: media
                        .1
                        .into_iter()
                        .map(|m| Url::from_str(m))
                        .collect::<Result<Vec<_>, _>>()?,
                    delivery: deliveries.clone(),
                    productno: v.sku,
                    //TODO: please parse it somehow...
                    description: product
                        .clone()
                        .description
                        .and_then(|d| Some(d.to_string())),
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
                });
            }
            Ok(shopitems)
        }
    }
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
}

#[derive(Clone, Debug)]
pub struct VariantUrl(url::Url);

impl VariantUrl {
    pub fn from_template<'a, T: Serialize>(
        url_template: String,
        context: &'a T,
    ) -> Result<Self, TryUrlFromTemplateError> {
        let mut tt = TinyTemplate::new();

        tt.add_template("t", &url_template)?;

        Ok(VariantUrl(url::Url::from_str(&tt.render("t", context)?)?))
    }
}

impl Deref for VariantUrl {
    type Target = url::Url;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for VariantUrl {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(thiserror::Error, Debug)]
pub enum TryUrlFromTemplateError {
    #[error("Something errorred during templating, {0}")]
    TinyTemplateError(#[from] tinytemplate::error::Error),
    #[error("failed parsing url created from urltemplate, {0}")]
    UrlParseError(#[from] url::ParseError),
}

pub async fn get_category_text_from_product(
    product: Product,
    saleor_api_url: &str,
    token: &str,
) -> Option<String> {
    if let Some(c) = product.category {
        match c.metafield {
            Some(val) => return Some(val),
            None => {
                let all_parents = get_category_parents(&c, saleor_api_url, token).await.ok();
                return all_parents
                    .map(|categories| categories.into_iter().find_map(|c| c.metafield))
                    .flatten();
            }
        }
    }
    None
}
