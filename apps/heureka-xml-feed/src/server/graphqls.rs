use super::event_handler::EventHandlerError;
use crate::queries::products_variants_categories::{
    Category, GetCategoryParent, GetCategoryParentVariables, GetProductsInitial,
    GetProductsInitialVariables, GetProductsNext, GetProductsNextVariables, Product,
};
use cynic::{QueryBuilder, http::SurfExt};
use tracing::{debug, error, info};

pub async fn get_all_products(
    saleor_api_url: &str,
    token: &str,
) -> Result<Vec<Product>, EventHandlerError> {
    let mut all_products = vec![];
    let res = surf::post(saleor_api_url)
        .header("authorization-bearer", token)
        .run_graphql(GetProductsInitial::build(GetProductsInitialVariables {
            channel: "",
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
        next_cursor = products.page_info.end_cursor;
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
                channel: "",
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

    let res = surf::post(saleor_api_url)
        .header("authorization-bearer", token)
        .run_graphql(GetCategoryParent::build(GetCategoryParentVariables {
            id: &category.id,
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
    todo!()
}
