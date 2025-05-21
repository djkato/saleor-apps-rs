use std::collections::HashMap;

use saleor_app_sdk::AuthData;
use serde::{Serialize, de::DeserializeOwned};
use surrealdb::{Surreal, engine::any::Any, opt::Resource};
use tracing::debug;

use crate::{
    queries::{
        products_variants_categories::{Category, Product, ProductVariant, ProductVariant2},
        query_shipping_details::ShippingZone,
    },
    server::{
        event_handler::MissingRelation,
        graphqls::{get_category_children, get_category_parents},
    },
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
    //TODO: add date to issues, make a proper table
    db.insert(Resource::from("issue")).content(issues).await?;
    Ok(())
}

fn fix_surreal_ids(id: &str) -> String {
    let mut new_id = id.replace("⟩", "").replace("⟨", "");
    let i = new_id.find(":");
    if let Some(i) = i {
        new_id.drain(..(i + 1));
    }
    new_id
}

pub fn surreal_value_to_types<T: DeserializeOwned>(
    data: surrealdb::Value,
) -> Result<Vec<T>, serde_json::Error> {
    let json = data.into_inner().into_json();
    let mut result: Vec<T> = vec![];
    if let Some(array) = json.as_array() {
        for mut val in array.to_owned() {
            if let Some(id) = val.get_mut("id")
                && let Some(str) = id.as_str()
            {
                let new_id = serde_json::to_value(fix_surreal_ids(str))?;
                *id = new_id;
            }

            result.push(serde_json::from_value(val)?);
        }
    } else {
        let mut new_json = json.clone();
        if let Some(id) = new_json.get_mut("id")
            && let Some(str) = id.as_str()
        {
            let new_id = serde_json::to_value(fix_surreal_ids(str))?;
            *id = new_id;
        }

        result.push(serde_json::from_value(new_json)?);
    }
    Ok(result)
}

pub async fn get_shipping_zones(
    db: &mut Surreal<Any>,
) -> Result<Vec<ShippingZone>, EventHandlerError> {
    Ok(surreal_value_to_types(
        db.query("SELECT * FROM shipping_zone").await?.take(0)?,
    )?)
}

pub async fn get_product_related_variants(
    db: &mut Surreal<Any>,
    product: &Product,
) -> Result<Vec<ProductVariant2>, EventHandlerError> {
    Ok(surreal_value_to_types(
        db.query(format!(
            "SELECT * FROM variant WHERE product:{}<-varies<-variant",
            product.id.inner().to_owned()
        ))
        .await?
        .take(0)?,
    )?)
}

pub async fn get_product_related_categories(
    db: &mut Surreal<Any>,
    product: &Product,
) -> Result<Vec<Category>, EventHandlerError> {
    let base_category: Vec<Category> = surreal_value_to_types(
        db.query(format!(
            "SELECT * FROM category WHERE product:{}<-categorises<-category",
            product.id.inner().to_owned()
        ))
        .await?
        .take(0)?,
    )?;

    let base_category = base_category
        .get(0)
        .ok_or(EventHandlerError::ProductMissingRelation(
            MissingRelation::Category,
        ))?;

    let mut all_categories = vec![base_category.clone()];

    let mut parent_category: Vec<Category> = surreal_value_to_types(
        db.query(format!(
            "SELECT * FROM category WHERE category:{}<-parents<-category",
            base_category.id.inner().to_owned()
        ))
        .await?
        .take(0)?,
    )?;

    while let Some(category) = parent_category.get(0) {
        all_categories.push(category.clone());

        parent_category = surreal_value_to_types(
            db.query(format!(
                "SELECT * FROM category WHERE category:{}<-parents<-category",
                category.id.inner().to_owned()
            ))
            .await?
            .take(0)?,
        )?;
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
            shipping_zone.id.inner().to_owned()
        )));
    }

    db.upsert(Resource::from("shipping_zone"))
        .content(serde_json::to_value(shipping_zone.clone())?)
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
    db.upsert(Resource::from("product"))
        .content(serde_json::to_value(product.clone())?)
        .await?;

    let category = product
        .clone()
        .category
        .ok_or(EventHandlerError::ProductMissingRelation(
            MissingRelation::Category,
        ))?;

    let all_category_parents =
        get_category_parents(&category, &ev.saleor_api_url, &token.token).await?;

    let all_category_children =
        get_category_children(&category, &ev.saleor_api_url, &token.token).await?;

    debug!(
        "inserting category {}:{} into db",
        &category.name,
        &category.id.inner()
    );
    db.upsert(Resource::from("category"))
        .content(serde_json::to_value(category.clone())?)
        .await?;

    clear_relations_categorises(db, category.id.inner()).await?;
    clear_relations_parents_in(db, category.id.inner()).await?;

    debug!(
        "relating category {}:{} -> categorises -> product {}:{}",
        &category.name,
        &category.id.inner(),
        &product.name,
        &product.id.inner()
    );

    db.query(format!(
        "RELATE category:`{}` -> categorises -> product:`{}`",
        category.id.inner().to_owned(),
        product.id.inner().to_owned()
    ))
    .await?;

    for parent in all_category_parents {
        debug!(
            "inserting category {}:{} into db",
            &parent.name,
            &parent.id.inner()
        );
        db.upsert(Resource::from("category"))
            .content(serde_json::to_value(category.clone())?)
            .await?;

        clear_relations_categorises(db, category.id.inner()).await?;

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
            "RELATE category:`{}` -> parents -> category:`{}`",
            parent.id.inner().to_owned(),
            category.id.inner().to_owned(),
        ))
        .await?;
    }

    for child in all_category_children {
        debug!(
            "inserting category {}:{} into db",
            &child.name,
            &child.id.inner()
        );
        db.upsert(Resource::from("category"))
            .content(serde_json::to_value(category.clone())?)
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
            "RELATE category:`{}` -> categorises -> product:`{}`",
            category.id.inner().to_owned(),
            product.id.inner().to_owned()
        ))
        .await?;

        clear_relations_parents_in(db, category.id.inner()).await?;
        clear_relations_parents_out(db, child.id.inner()).await?;

        debug!(
            "relating category {}:{} -> childs -> category {}:{}",
            &category.name,
            &category.id.inner(),
            &child.name,
            &child.id.inner()
        );

        db.query(format!(
            "RELATE category:`{}` -> childs -> category:`{}`",
            child.id.inner().to_owned(),
            category.id.inner().to_owned(),
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
    db.upsert(Resource::from("variant"))
        .content(serde_json::to_value(variant.clone())?)
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
        "RELATE variant:`{}` -> varies -> product:`{}`",
        variant.id.inner().to_owned(),
        product.id.inner().to_owned()
    ))
    .await?;
    Ok(())
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
    debug!("Clearing all category:{}->categorises", cat_id);
    db.query("DELETE (SELECT * FROM categorises WHERE out = $cat_id);")
        .bind(("cat_id", cat_id.to_owned()))
        .await?;
    Ok(())
}

pub async fn clear_relations_parents_in(
    db: &mut Surreal<Any>,
    cat_id: &str,
) -> Result<(), surrealdb::Error> {
    debug!("Clearing all category:{}<-parents", cat_id);
    db.query("DELETE (SELECT * FROM parents WHERE in = $cat_id);")
        .bind(("cat_id", cat_id.to_owned()))
        .await?;
    Ok(())
}

pub async fn clear_relations_parents_out(
    db: &mut Surreal<Any>,
    cat_id: &str,
) -> Result<(), surrealdb::Error> {
    debug!("Clearing all category:{}->parents", cat_id);
    db.query("DELETE (SELECT * FROM parents WHERE out = $cat_id);")
        .bind(("cat_id", cat_id.to_owned()))
        .await?;
    Ok(())
}
