use std::{
    ops::{Deref, DerefMut},
    str::FromStr,
};

use heureka_xml_feed::{Delivery, ShopItem};
use rust_decimal::{Decimal, prelude::FromPrimitive};
use serde::{Serialize, de::DeserializeOwned};
use tinytemplate::TinyTemplate;
use url::Url;

use crate::{
    app::AppSettings,
    queries::event_products_updated::{Category, Product2, ProductVariant},
};

pub mod task_handler;

pub fn try_shopitem_from_variant<'a, T: Serialize>(
    v: ProductVariant,
    deliveries: Vec<Delivery>,
    url_template: String,
    url_context: &T,
    settings: AppSettings,
) -> Result<ShopItem, TryIntoShopItemError> {
    let media = v
        .media
        .clone()
        .and_then(|m| Some(m.into_iter().map(|u| u.url).collect::<Vec<_>>()))
        .ok_or(TryIntoShopItemError::MissingVariants)?;
    let media = media
        .split_first()
        .ok_or(TryIntoShopItemError::MissingVariants)?;
    Ok(ShopItem {
        item_id: v.id.into_inner(),
        url: Some(VariantUrl::from_template(url_template.clone(), url_context)?.0),
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
        //I don't care enough to parse it :)
        description: v.product.clone().description.and_then(|d| Some(d.0)),
        categorytext: get_category_text_from_product(Product2 {
            variants: None,
            name: v.product.name,
            description: None,
            id: v.product.id,
            category: v.product.category,
        })
        .unwrap_or("".to_owned()),
        accessory: vec![],
        param: vec![],
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
    })
}

pub fn try_shopitem_from_product<'a, T: Serialize>(
    product: Product2,
    deliveries: Vec<Delivery>,
    url_template: String,
    url_context: &T,
    settings: AppSettings,
) -> Result<Vec<ShopItem>, TryIntoShopItemError> {
    match product.clone().variants {
        None => Err(TryIntoShopItemError::MissingVariants),
        Some(variants) => {
            let mut shopitems: Vec<ShopItem> = vec![];
            for v in variants.into_iter() {
                let media = v
                    .media
                    .clone()
                    .and_then(|m| Some(m.into_iter().map(|u| u.url).collect::<Vec<_>>()))
                    .ok_or(TryIntoShopItemError::MissingVariants)?;
                let media = media
                    .split_first()
                    .ok_or(TryIntoShopItemError::MissingVariants)?;
                shopitems.push(ShopItem {
                    item_id: v.id.into_inner(),
                    url: Some(VariantUrl::from_template(url_template.clone(), url_context)?.0),
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
                    //I don't care enough to parse it :)
                    description: product.clone().description.and_then(|d| Some(d.0)),
                    categorytext: get_category_text_from_product(product.clone())
                        .unwrap_or("".to_owned()),
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

pub fn get_category_text_from_product<T: Serialize + DeserializeOwned>(
    product: T,
) -> Option<String> {
    let product: Product2 = serde_json::from_str(&serde_json::to_string(&product).ok()?).ok()?;

    if let Some(c) = product.category {
        match c.metafield {
            Some(val) => return Some(val),
            None => return go_deeper(c).ok().flatten(),
        }
    }
    None
}

fn go_deeper<C: Serialize + DeserializeOwned>(c: C) -> Result<Option<String>, serde_json::Error> {
    let c: Category = serde_json::from_str(&serde_json::to_string(&c)?)?;
    match c.parent {
        None => go_deeper(c),
        Some(c) => match c.metafield {
            Some(meta) => Ok(Some(meta)),
            None => go_deeper(c),
        },
    }
}
// product
//     .category
//     .and_then(|c|
//         c.metafield.or_else(||
//             c.parent.and_then(|c|
//                 c.metafield.or_else(||
//                     c.parent.and_then(|c|
//                         c.metafield.or_else(||
//                             c.parent.and_then(|c|
//                                 c.metafield.or_else(||
//                                     c.parent.and_then(|c|
//                                         c.metafield.or_else(||
//                                             c.parent.and_then(|c|
//                                                 c.metafield.or_else(||
//                                                     c.parent.and_then(|c|
//                                                         c.metafield.or_else(||
//                                                             c.parent.and_then(|c|
//                                                                 c.metafield.or_else(||
//                                                                     c.parent.and_then(|c|
//                                                                         c.metafield.or_else(||
//                                                                             c.parent.and_then(|c|
//                                                                                 c.metafield.or_else(||
//                                                                                     c.parent.and_then(|c|
//                                                                                         c.metafield.or_else(||
//                                                                                             c.parent.and_then(|c|
//                                                                                                 c.metafield.or_else(||
//                                                                                                     c.parent.and_then(|c|
//                                                                                                         c.metafield
//                                                                                                     )
//                                                                                                 )
//                                                                                             )
//                                                                                         )
//                                                                                     )
//                                                                                 )
//                                                                             )
//                                                                         )
//                                                                     )
//                                                                 )
//                                                             )
//                                                         )
//                                                     )
//                                                 )
//                                             )
//                                         )
//                                     )
//                                 )
//                             )
//                         )
//                     )
//                 )
//             )
//         )
//     )
