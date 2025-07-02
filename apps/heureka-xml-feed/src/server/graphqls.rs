use super::event_handler::EventHandlerError;
use crate::queries::{
    products_variants_categories::{
        Category, Category2, GetCategoryChildren, GetCategoryChildrenVariables, GetCategoryParent,
        GetCategoryParentVariables, GetProductsInitial, GetProductsInitialVariables,
        GetProductsNext, GetProductsNextVariables, Product,
    },
    query_shipping_details::{DefaultShippingZone, DefaultShippingZoneVariables, ShippingZone},
};
use cynic::{QueryBuilder, http::SurfExt};
use tracing::{debug, error, info};

pub async fn get_shipping_zones(
    saleor_api_url: &str,
    token: &str,
    channel: &str,
    shipping_zone_ids: &Vec<String>,
) -> Result<Vec<ShippingZone>, EventHandlerError> {
    let mut zones = vec![];
    for id in shipping_zone_ids {
        let res = surf::post(saleor_api_url)
            .header("authorization-bearer", token)
            .run_graphql(DefaultShippingZone::build(DefaultShippingZoneVariables {
                channel,
                id: &cynic::Id::new(id.clone()),
            }))
            .await?;

        if let Some(e) = res.errors {
            for error in &e {
                error!("Errors during graphql, {:?}", error.message);
            }
            for error in e {
                return Err(error.into());
            }
        }

        if let Some(data) = res.data
            && let Some(zone) = data.shipping_zone
        {
            zones.push(zone);
        }
    }
    Ok(zones)
}

pub async fn get_all_products(
    saleor_api_url: &str,
    channel: &str,
    token: &str,
) -> Result<Vec<Product>, EventHandlerError> {
    let mut all_products = vec![];
    let res = surf::post(saleor_api_url)
        .header("authorization-bearer", token)
        .run_graphql(GetProductsInitial::build(GetProductsInitialVariables {
            channel,
        }))
        .await?;

    if let Some(e) = res.errors {
        for error in &e {
            error!("Errors during graphql, {:?}", error.message);
        }
        for error in e {
            return Err(error.into());
        }
    }

    let mut next_cursor = None;

    if let Some(products_initial) = res.data
        && let Some(products) = products_initial.products
    {
        all_products.append(
            &mut products
                .edges
                .into_iter()
                .map(|p| p.node)
                .collect::<Vec<_>>(),
        );
        if products.page_info.has_next_page {
            next_cursor = products.page_info.end_cursor;
        }
    }

    debug!(
        "collected first {} products, is there more? {}",
        all_products.len(),
        next_cursor.is_some()
    );

    while let Some(cursor) = &mut next_cursor {
        let res = surf::post(saleor_api_url)
            .header("authorization-bearer", token)
            .run_graphql(GetProductsNext::build(GetProductsNextVariables {
                after: cursor,
                channel,
            }))
            .await?;

        if let Some(e) = res.errors {
            for error in &e {
                error!("Errors during graphql, {:?}", error.message);
            }
            for error in e {
                return Err(error.into());
            }
        }

        if let Some(products_next) = res.data
            && let Some(products) = products_next.products
        {
            all_products.append(
                &mut products
                    .edges
                    .into_iter()
                    .map(|p| p.node)
                    .collect::<Vec<_>>(),
            );
            next_cursor = products.page_info.end_cursor;
        }

        debug!(
            "collected {} products, is there more? {}",
            all_products.len(),
            next_cursor.is_some()
        );
    }
    info!("collected a total of {} products", all_products.len());
    Ok(all_products)
}

///Returns parent.parent.parent.... in order, excluding first product.category
pub async fn get_category_parents(
    category: &Category,
    saleor_api_url: &str,
    token: &str,
) -> Result<Vec<Category>, EventHandlerError> {
    debug!(
        "Collecting all parent categories of category {}:{}",
        category.name,
        &category.id.inner(),
    );

    let mut all_parents = vec![];
    let mut parent = Some(Category2 {
        name: category.name.clone(),
        id: category.id.clone(),
        slug: category.slug.clone(),
        metafield: category.metafield.clone(),
    });

    while let Some(curr_category) = parent {
        let res = surf::post(saleor_api_url)
            .header("authorization-bearer", token)
            .run_graphql(GetCategoryParent::build(GetCategoryParentVariables {
                id: &curr_category.clone().id,
            }))
            .await?;

        if let Some(e) = res.errors {
            for error in &e {
                error!("Errors during graphql, {:?}", error.message);
            }
            for error in e {
                return Err(error.into());
            }
        }
        if let Some(data) = res.data
            && let Some(category) = data.category
        {
            all_parents.push(category.clone());
            parent = category.parent;
            continue;
        }
        parent = None;
    }
    debug!("Collected {} parent categories", all_parents.len());
    Ok(all_parents)
}

pub async fn get_category_children(
    category: &Category,
    saleor_api_url: &str,
    token: &str,
) -> Result<Vec<Category>, EventHandlerError> {
    debug!(
        "Collecting all child categories of category {}:{}",
        category.name,
        &category.id.inner(),
    );

    let mut all_children = vec![];
    let res = surf::post(saleor_api_url)
        .header("authorization-bearer", token)
        .run_graphql(GetCategoryChildren::build(GetCategoryChildrenVariables {
            id: &category.clone().id,
            after: None,
        }))
        .await?;
    if let Some(e) = res.errors {
        for error in &e {
            error!("Errors during graphql, {:?}", error.message);
        }
        for error in e {
            return Err(error.into());
        }
    }

    let mut after_cursor = None;

    if let Some(data) = res.data
        && let Some(category) = data.category
        && let Some(children) = category.children
    {
        if children.page_info.has_next_page {
            after_cursor = children.page_info.end_cursor;
        }
        all_children.append(
            &mut children
                .edges
                .iter()
                .map(|c| c.node.clone())
                .collect::<Vec<_>>(),
        );
    }

    debug!(
        "collected first {} child categories, is there more? {}",
        all_children.len(),
        after_cursor.is_some()
    );

    while let Some(after) = after_cursor {
        let res = surf::post(saleor_api_url)
            .header("authorization-bearer", token)
            .run_graphql(GetCategoryChildren::build(GetCategoryChildrenVariables {
                id: &category.clone().id,
                after: Some(&after),
            }))
            .await?;

        if let Some(e) = res.errors {
            for error in &e {
                error!("Errors during graphql, {:?}", error.message);
            }
            for error in e {
                return Err(error.into());
            }
        }

        after_cursor = None;

        if let Some(data) = res.data
            && let Some(category) = data.category
            && let Some(children) = category.children
        {
            if children.page_info.has_next_page {
                after_cursor = children.page_info.end_cursor;
            }
            all_children.append(
                &mut children
                    .edges
                    .iter()
                    .map(|c| c.node.clone())
                    .collect::<Vec<_>>(),
            );
        }

        debug!(
            "collected more {} child categories, is there more? {}",
            all_children.len(),
            after_cursor.is_some()
        );
    }

    debug!("Collected {} child categories", all_children.len());
    Ok(all_children)
}
