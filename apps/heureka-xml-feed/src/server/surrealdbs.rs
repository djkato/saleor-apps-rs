use saleor_app_sdk::AuthData;
use surrealdb::{Surreal, engine::any::Any};
use tracing::debug;

use crate::{
    queries::{
        products_variants_categories::{Category, Product, ProductVariant, ProductVariant2},
        query_shipping_details::ShippingZone,
    },
    server::{event_handler::MissingRelation, graphqls::get_category_parents},
};

use super::event_handler::{EventHandlerError, RegenerateEvent};

pub fn into_surreal_id(str: String) -> String {
    str.chars().filter(|c| c.is_alphanumeric()).collect()
}

pub async fn save_issues(
    db: &mut Surreal<Any>,
    e: Vec<EventHandlerError>,
) -> Result<(), EventHandlerError> {
    let issues = e.into_iter().map(|e| e.to_string()).collect::<Vec<_>>();
    let _: Vec<String> = db.insert("issue").content(issues).await?;
    Ok(())
}

pub async fn get_shipping_zones(
    db: &mut Surreal<Any>,
) -> Result<Vec<ShippingZone>, EventHandlerError> {
    Ok(db.select("shipping_zone").await?)
}

pub async fn get_product_related_variants(
    db: &mut Surreal<Any>,
    product: &Product,
) -> Result<Vec<ProductVariant2>, EventHandlerError> {
    let variants: Vec<ProductVariant2> = db
        .query(format!(
            "SELECT * FROM variant WHERE product:{}<-varies<-variant",
            into_surreal_id(product.id.inner().to_owned())
        ))
        .await?
        .take(0)?;
    Ok(variants)
}

pub async fn get_product_related_categories(
    db: &mut Surreal<Any>,
    product: &Product,
) -> Result<Vec<Category>, EventHandlerError> {
    let base_category: Option<Category> = db
        .query(format!(
            "SELECT * FROM category WHERE product:{}<-categorises<-category LIMIT 1",
            into_surreal_id(product.id.inner().to_owned())
        ))
        .await?
        .take(0)?;

    let base_category = base_category.ok_or(EventHandlerError::ProductMissingRelation(
        MissingRelation::Category,
    ))?;

    let mut all_categories = vec![base_category.clone()];

    let mut parent_category: Option<Category> = db
        .query(format!(
            "SELECT * FROM category WHERE category:{}<-parents<-category",
            into_surreal_id(base_category.id.inner().to_owned())
        ))
        .await?
        .take(0)?;

    while let Some(category) = parent_category {
        all_categories.push(category.clone());

        parent_category = db
            .query(format!(
                "SELECT * FROM category WHERE category:{}<-parents<-category",
                into_surreal_id(category.id.inner().to_owned())
            ))
            .await?
            .take(0)?;
    }

    Ok(all_categories)
}

pub async fn save_shipping_zone_to_db(
    shipping_zone: &ShippingZone,
    db: &mut Surreal<Any>,
) -> Result<(), EventHandlerError> {
    debug!(
        "inserting shipping zone {}:{:?} into db",
        shipping_zone.id.inner(),
        shipping_zone.metafield
    );

    if shipping_zone.metafield.is_none() {
        return Err(EventHandlerError::ShippingZoneMisconfigured(format!(
            "Shipping zone {} is missing metadata 'heureka_courierid'",
            shipping_zone.id.inner()
        )));
    }

    let _: Option<ShippingZone> = db
        .upsert((
            "shipping_zone",
            into_surreal_id(shipping_zone.id.inner().to_owned()),
        ))
        .content(shipping_zone.clone())
        .await?;

    Ok(())
}

pub async fn save_product_and_category_to_db(
    product: &Product,
    ev: &RegenerateEvent,
    token: &AuthData,
    db: &mut Surreal<Any>,
) -> Result<(), EventHandlerError> {
    debug!(
        "inserting product {}:{} into db",
        &product.name,
        &product.id.inner()
    );
    let _: Option<Product> = db
        .upsert(("product", into_surreal_id(product.id.inner().to_owned())))
        .content(product.clone())
        .await?;

    let category = product
        .clone()
        .category
        .ok_or(EventHandlerError::ProductMissingRelation(
            MissingRelation::Category,
        ))?;

    let all_category_parents =
        get_category_parents(&category, &ev.saleor_api_url, &token.token).await?;

    for parent in all_category_parents {
        debug!(
            "inserting category {}:{} into db",
            &parent.name,
            &parent.id.inner()
        );
        let _: Option<Category> = db
            .upsert(("category", into_surreal_id(category.id.inner().to_owned())))
            .content(category.clone())
            .await?;

        clear_relations_categorises(db, category.id.inner()).await?;

        debug!(
            "relating category {}:{} -> categorises -> product {}:{}",
            &category.name,
            &category.id.inner(),
            &product.name,
            &product.id.inner()
        );

        db.query(format!(
            "RELATE category:{}->categorises->product:{}",
            into_surreal_id(category.id.inner().to_owned()),
            into_surreal_id(product.id.inner().to_owned())
        ))
        .await?;

        clear_relations_parents_in(db, category.id.inner()).await?;
        clear_relations_parents_out(db, parent.id.inner()).await?;

        debug!(
            "relating category {}:{} -> parents -> category {}:{}",
            &category.name,
            &category.id.inner(),
            &parent.name,
            &parent.id.inner()
        );

        db.query(format!(
            "RELATE category(parent):{} -> parents -> category:{}",
            into_surreal_id(parent.id.inner().to_owned()),
            into_surreal_id(category.id.inner().to_owned()),
        ))
        .await?;
    }
    Ok(())
}

pub async fn save_variants_to_db(
    variant: &ProductVariant2,
    db: &mut Surreal<Any>,
    product: &Product,
) -> Result<(), EventHandlerError> {
    debug!(
        "inserting variant {}:{}",
        &variant.name,
        &variant.id.inner(),
    );
    let _: Option<ProductVariant> = db
        .upsert(("variant", into_surreal_id(variant.id.inner().to_owned())))
        .content(variant.clone())
        .await?;

    clear_relations_varies(db, variant.id.inner()).await?;

    debug!(
        "relating variant {}:{} -> varies -> product {}:{}",
        &variant.name,
        &variant.id.inner(),
        &product.name,
        &product.id.inner()
    );

    db.query(format!(
        "RELATE variant:{}->varies->product:{}",
        into_surreal_id(variant.id.inner().to_owned()),
        into_surreal_id(product.id.inner().to_owned()),
    ))
    .await?;
    Ok(())
}

pub async fn clear_relations_varies(
    db: &mut Surreal<Any>,
    var_id: &str,
) -> Result<(), surrealdb::Error> {
    db.query("DELETE (SELECT * FROM varies WHERE in = $var_id);")
        .bind(("var_id", into_surreal_id(var_id.to_owned())))
        .await?;
    Ok(())
}

pub async fn clear_relations_categorises(
    db: &mut Surreal<Any>,
    cat_id: &str,
) -> Result<(), surrealdb::Error> {
    db.query("DELETE (SELECT * FROM categorises WHERE in = $cat_id);")
        .bind(("cat_id", into_surreal_id(cat_id.to_owned())))
        .await?;
    Ok(())
}

pub async fn clear_relations_parents_in(
    db: &mut Surreal<Any>,
    cat_id: &str,
) -> Result<(), surrealdb::Error> {
    db.query("DELETE (SELECT * FROM parents WHERE in = $cat_id);")
        .bind(("cat_id", into_surreal_id(cat_id.to_owned())))
        .await?;
    Ok(())
}

pub async fn clear_relations_parents_out(
    db: &mut Surreal<Any>,
    cat_id: &str,
) -> Result<(), surrealdb::Error> {
    db.query("DELETE (SELECT * FROM parents WHERE out = $cat_id);")
        .bind(("cat_id", into_surreal_id(cat_id.to_owned())))
        .await?;
    Ok(())
}
