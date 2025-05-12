use std::{
    ops::{Deref, DerefMut},
    str::FromStr,
};

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

pub async fn try_shop_from_db(
    db: Surreal<Any>,
    deliveries: Vec<Delivery>,
    url_template: String,
    settings: AppSettings,
) -> Result<Shop, TryIntoShopError> {
    let products: Vec<Product> = db.select("product").await?;

    let mut shopitems: Vec<ShopItem> = vec![];
    for mut product in products {
        let variants: Vec<ProductVariant2> = db
            .query(format!(
                "SELECT * FROM variant WHERE product:{}<-varies<-variant",
                product.id.inner().to_owned()
            ))
            .await?
            .take(0)?;

        let categories: Vec<Category> = db
            .query(format!(
                "SELECT * FROM category WHERE product:{}<-categorises<-category LIMIT 1",
                product.id.inner().to_owned()
            ))
            .await?
            .take(0)?;

        let mut category = categories.into_iter().nth(0);

        if let Some(base_category) = &mut category {
            let mut parent_category: Option<Category> = db
                .query(format!(
                    "SELECT * FROM category WHERE category:{}<-parents<-category",
                    base_category.id.inner().to_owned()
                ))
                .await?
                .take(0)?;

            while let Some(category) = parent_category {
                parent_category = db
                    .query(format!(
                        "SELECT * FROM category WHERE category:{}<-parents<-category",
                        category.id.inner().to_owned()
                    ))
                    .await?
                    .take(0)?;
            }
        }
        // variants and categories that are present with product in db aren't being updated, only
        // the tables are. Just cba to strip the db of these parts
        product.category = category;
        product.variants = match variants.is_empty() {
            true => None,
            false => Some(variants),
        };
        shopitems.append(&mut try_shopitem_from_product(
            product.clone(),
            deliveries.clone(),
            url_template.clone(),
            &product,
            settings.clone(),
        )?);
    }

    todo!()
}
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
        itemgroup_id: Some(v.product.id.inner().to_owned()),
        manufacturer: None,
        product: None,
        special_service: vec![],
        video_url: None,
        categorytext: get_category_text_from_product(Product {
            variants: None,
            name: v.product.name,
            description: None,
            id: v.product.id,
            category: v.product.category,
        })
        .unwrap_or("".to_owned()),
    })
}

pub fn try_shopitem_from_product<'a, T: Serialize>(
    product: Product,
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

pub fn get_category_text_from_product(product: Product) -> Option<String> {
    if let Some(c) = product.category {
        match c.metafield {
            Some(val) => return Some(val),
            //TODO: Create a while loop query for parent category till maybe one is found
            None => return None,
        }
    }
    None
}

pub async fn clear_relations_varies(
    db: &mut Surreal<Any>,
    var_id: &str,
) -> Result<(), surrealdb::Error> {
    db.query("DELETE (SELECT * FROM varies WHERE in = $var_id);")
        .bind(("var_id", var_id.to_owned()))
        .await?;
    Ok(())
}

pub async fn clear_relations_categorises(
    db: &mut Surreal<Any>,
    cat_id: &str,
) -> Result<(), surrealdb::Error> {
    db.query("DELETE (SELECT * FROM categorises WHERE in = $cat_id);")
        .bind(("cat_id", cat_id.to_owned()))
        .await?;
    Ok(())
}

pub async fn clear_relations_parents_in(
    db: &mut Surreal<Any>,
    cat_id: &str,
) -> Result<(), surrealdb::Error> {
    db.query("DELETE (SELECT * FROM parents WHERE in = $cat_id);")
        .bind(("cat_id", cat_id.to_owned()))
        .await?;
    Ok(())
}

pub async fn clear_relations_parents_out(
    db: &mut Surreal<Any>,
    cat_id: &str,
) -> Result<(), surrealdb::Error> {
    db.query("DELETE (SELECT * FROM parents WHERE out = $cat_id);")
        .bind(("cat_id", cat_id.to_owned()))
        .await?;
    Ok(())
}
