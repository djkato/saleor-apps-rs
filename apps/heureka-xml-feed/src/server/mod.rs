use std::{
    ops::{Deref, DerefMut},
    str::FromStr,
};

use heureka_xml_feed::ShopItem;
use rust_decimal::{prelude::FromPrimitive, Decimal};
use serde::Serialize;
use tinytemplate::TinyTemplate;
use url::Url;

use crate::{app::AppSettings, queries::event_products_updated::Product2};

pub mod task_handler;

fn try_shopitem_from_product<'a, T: Serialize>(
    product: Product2,
    url_template: String,
    url_context: &T,
    settings: AppSettings,
) -> Result<Vec<ShopItem>, TryIntoShopItemError> {
    match product.variants {
        None => Err(TryIntoShopItemError::MissingVariants),
        Some(variants) => {
            for v in variants.into_iter() {
                let media = v
                    .media
                    .and_then(|m| {
                        m.into_iter()
                            .map(|u| u.url)
                            .collect::<Vec<_>>()
                            .split_first()
                    })
                    .ok_or(TryIntoShopItemError::MissingVariants)?;

                ShopItem {
                    item_id: v.id.into_inner(),
                    url: Some(VariantUrl::from_template(url_template, url_context)?.0),
                    productname: v.name,
                    product: None,
                    price_vat: v
                        .pricing
                        .and_then(|p| {
                            p.price
                                .and_then(|p| Decimal::from_f32(p.gross.amount as f32))
                        })
                        .ok_or(TryIntoShopItemError::PricingMissingOrFailed)?,
                    vat: Some(settings.tax_rate),
                    ean: None,
                    isbn: None,
                    dues: None,
                    gift: None,
                    param: vec![],
                    imgurl: Url::from_str(media.0)?,
                    imgurl_alternative: media
                        .1
                        .into_iter()
                        .map(|m| Url::from_str(m))
                        .collect::<Result<Vec<_>, _>>()?,
                    gift_id: None,
                    delivery: settings.delivery_types,
                    video_url: None,
                    item_type: None,
                    productno: v.sku,
                    accessory: None,
                    description: product.description,
                    heureka_cpc: None,
                    manufacturer: None,
                    // categorytext:
                }
            }
        }
    }
    todo!()
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
    fn from_template<'a, T: Serialize>(
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

fn get_category_text_from_product(product: &Product2) -> Option<String> {
    product.product_type.metafield.or_else(|| {
        product.category.and_then(|first_c| {
            let mut c = Box::new(first_c);
            while let Some(c) = c.parent {
                if let Some(metafield) = c.metafield {
                    return Some(metafield);
                }
                if let Some(parent) = c.parent {
                    c = parent;
                } else {
                    break;
                };
            }
            None
        })
    })
}

trait HasMaybeParentAndMetafield {
    fn get_metafield(self) -> Option<String>;
    fn get_parent(self) -> Option<impl HasMaybeParentAndMetafield>;
}
