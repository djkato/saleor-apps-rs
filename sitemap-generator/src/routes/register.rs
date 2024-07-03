use std::{str::FromStr, sync::Arc};

use anyhow::Context;
use axum::{
    extract::Json,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use cynic::{http::SurfExt, QueryBuilder};
use saleor_app_sdk::{AuthData, AuthToken};
use tinytemplate::TinyTemplate;
use tokio::spawn;
use tracing::{debug, error, info, trace};

use crate::{
    app::{AppError, AppState},
    queries::{
        event_subjects_updated::{
            self, CategoryUpdated, CollectionUpdated, PageUpdated, ProductUpdated,
        },
        get_all_categories_n_products::{
            CategorisedProduct, Category3, GetCategoriesInitial, GetCategoriesNext,
            GetCategoriesNextVariables, GetCategoryProductsInitial,
            GetCategoryProductsInitialVariables, GetCategoryProductsNext,
            GetCategoryProductsNextVariables,
        },
        get_all_collections::{
            Collection, GetCollectionsInitial, GetCollectionsNext, GetCollectionsNextVariables,
        },
        get_all_pages::{self, GetPagesInitial, GetPagesNext, GetPagesNextVariables},
    },
};

pub async fn register(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(auth_token): Json<AuthToken>,
) -> Result<StatusCode, AppError> {
    debug!(
        "/api/register:\nsaleor_api_url:{:?}\nauth_token:{:?}",
        headers.get("saleor-api-url"),
        auth_token
    );

    if auth_token.auth_token.is_empty() {
        return Err(anyhow::anyhow!("missing auth_token").into());
    }
    let app = state.saleor_app.lock().await;
    let saleor_api_url = headers.get("saleor-api-url").context("missing api field")?;
    let saleor_api_url = saleor_api_url.to_str()?.to_owned();
    let auth_data = AuthData {
        jwks: None,
        token: auth_token.auth_token,
        domain: Some(state.config.app_api_base_url.clone()),
        app_id: state.manifest.id.clone(),
        saleor_api_url: saleor_api_url.clone(),
    };
    app.apl.set(auth_data).await?;

    info!("registered app for{:?}", &saleor_api_url);

    //When app registers, start collecting everything of substance
    info!("Starting caching and generation process");
    let cloned_state = state.clone();
    spawn(async move {
        match regenerate(cloned_state, saleor_api_url).await {
            Ok(_) => info!("Finished caching and regeneration"),
            Err(e) => error!("Something went wrong during caching and regeneration, {e}"),
        };
    });
    Ok(StatusCode::OK)
}

pub async fn regenerate(state: AppState, saleor_api_url: String) -> anyhow::Result<()> {
    info!("regeneration: fetching all categories, products, collections, pages");
    let app = state.saleor_app.lock().await;
    let auth_data = app.apl.get(&saleor_api_url).await?;

    let pages = get_all_pages(&saleor_api_url, &auth_data.token).await?;
    let collections = get_all_collections(&saleor_api_url, &auth_data.token).await?;
    info!(
        "regeneration: found {} products, {} categories, {} pages, {} collections",
        0,
        0,
        pages.len(),
        collections.len()
    );
    info!("regeneration: creating xml data");
    info!("regeneration: creating urls");
    // write_xml(page_urls, &state, XmlDataType::Page).await?;
    // write_xml(collection_urls, &state, XmlDataType::Collection).await?;
    // write_xml(category_urls, &state, XmlDataType::Category).await?;
    // write_xml(product_urls, &state, XmlDataType::Product).await?;
    Ok(())
}

async fn get_all_pages(
    saleor_api_url: &str,
    token: &str,
) -> anyhow::Result<Vec<get_all_pages::Page>> {
    let operation = GetPagesInitial::build(());
    let mut all_pages = vec![];
    let res = surf::post(saleor_api_url)
        .header("authorization-bearer", token)
        .run_graphql(operation)
        .await;
    if let Ok(query) = &res
        && let Some(data) = &query.data
        && let Some(pages) = &data.pages
    {
        debug!("fetched first pages, eg.:{:?}", &pages.edges.first());
        all_pages.append(
            &mut pages
                .edges
                .iter()
                .map(|p| p.node.clone())
                .collect::<Vec<_>>(),
        );
        //Keep fetching next page
        let mut next_cursor = pages.page_info.end_cursor.clone();
        while let Some(cursor) = &mut next_cursor {
            let res = surf::post(saleor_api_url)
                .header("authorization-bearer", token)
                .run_graphql(GetPagesNext::build(GetPagesNextVariables { after: cursor }))
                .await;
            if let Ok(query) = &res
                && let Some(data) = &query.data
                && let Some(pages) = &data.pages
            {
                all_pages.append(
                    &mut pages
                        .edges
                        .iter()
                        .map(|p| p.node.clone())
                        .collect::<Vec<_>>(),
                );
                debug!("fetched next pages, eg.:{:?}", &pages.edges.first());
                next_cursor.clone_from(&pages.page_info.end_cursor);
            } else {
                error!("Failed fetching next pages! {:?}", &res);
                anyhow::bail!("Failed fetching next pages! {:?}", res);
            }
        }
    } else {
        error!("Failed fetching initial pages! {:?}", &res);
        anyhow::bail!("Failed fetching initial pages! {:?}", res);
    };
    info!("fetched all pages");
    Ok(all_pages)
}

async fn get_all_categories(saleor_api_url: &str, token: &str) -> anyhow::Result<Vec<Category3>> {
    debug!("Collecting all categories...");
    let operation = GetCategoriesInitial::build(());
    let mut all_categories = vec![];
    let res = surf::post(saleor_api_url)
        .header("authorization-bearer", token)
        .run_graphql(operation)
        .await;
    if let Ok(query) = &res
        && let Some(data) = &query.data
        && let Some(categories) = &data.categories
    {
        all_categories.append(
            &mut categories
                .edges
                .iter()
                .map(|p| p.node.clone())
                .collect::<Vec<_>>(),
        );
        debug!(
            "fetched first categories, eg.:{:?}",
            &categories.edges.first()
        );
        //Keep fetching next page
        let mut next_cursor = categories.page_info.end_cursor.clone();
        while let Some(cursor) = &mut next_cursor {
            let res = surf::post(saleor_api_url)
                .header("authorization-bearer", token)
                .run_graphql(GetCategoriesNext::build(GetCategoriesNextVariables {
                    after: Some(cursor),
                }))
                .await;
            if let Ok(query) = &res
                && let Some(data) = &query.data
                && let Some(categories) = &data.categories
            {
                all_categories.append(
                    &mut categories
                        .edges
                        .iter()
                        .map(|p| p.node.clone())
                        .collect::<Vec<_>>(),
                );
                debug!(
                    "fetched next categories, eg.:{:?}",
                    &categories.edges.first()
                );
                next_cursor.clone_from(&categories.page_info.end_cursor);
            } else {
                error!("Failed fetching next categories! {:?}", &res);
                anyhow::bail!("Failed fetching next categories! {:?}", res);
            }
        }
    } else {
        error!("Failed fetching initial Categories! {:?}", &res);
        anyhow::bail!("Failed fetching initial Categories! {:?}", res);
    };
    info!("All categories collected");
    Ok(all_categories)
}

async fn get_all_collections(saleor_api_url: &str, token: &str) -> anyhow::Result<Vec<Collection>> {
    debug!("Collecting all Collections...");
    let operation = GetCollectionsInitial::build(());
    let mut all_collections = vec![];
    let res = surf::post(saleor_api_url)
        .header("authorization-bearer", token)
        .run_graphql(operation)
        .await;
    if let Ok(query) = &res
        && let Some(data) = &query.data
        && let Some(collections) = &data.collections
    {
        all_collections.append(
            &mut collections
                .edges
                .iter()
                .map(|p| p.node.clone())
                .collect::<Vec<_>>(),
        );
        debug!(
            "fetched first collections, eg.:{:?}",
            &collections.edges.first()
        );

        //Keep fetching next page
        let mut next_cursor = collections.page_info.end_cursor.clone();
        while let Some(cursor) = &mut next_cursor {
            let res = surf::post(saleor_api_url)
                .header("authorization-bearer", token)
                .run_graphql(GetCollectionsNext::build(GetCollectionsNextVariables {
                    after: Some(cursor),
                }))
                .await;
            if let Ok(query) = &res
                && let Some(data) = &query.data
                && let Some(collections) = &data.collections
            {
                all_collections.append(
                    &mut collections
                        .edges
                        .iter()
                        .map(|p| p.node.clone())
                        .collect::<Vec<_>>(),
                );
                debug!(
                    "fetched next collections, eg.:{:?}",
                    &collections.edges.first()
                );
                next_cursor.clone_from(&collections.page_info.end_cursor);
            } else {
                error!("Failed fetching next collecnios! {:?}", &res);
                anyhow::bail!("Failed fetching next collections! {:?}", res);
            }
        }
    } else {
        error!("Failed fetching initial collections! {:?}", &res);
        anyhow::bail!("Failed fetching initial collections! {:?}", res);
    };
    info!("All Collections collected...");
    Ok(all_collections)
}
/**
 * Gets all products of a category then assings them as related
 */
async fn get_all_products(
    saleor_api_url: &str,
    channel: &str,
    token: &str,
    main_category: &mut (Category3, Vec<Arc<CategorisedProduct>>),
) -> anyhow::Result<Vec<Arc<CategorisedProduct>>> {
    debug!("Collecting all products...");
    let operation = GetCategoryProductsInitial::build(GetCategoryProductsInitialVariables {
        id: &main_category.0.id,
        channel,
    });
    let mut all_categorised_products: Vec<Arc<CategorisedProduct>> = vec![];
    let res = surf::post(saleor_api_url)
        .header("authorization-bearer", token)
        .run_graphql(operation)
        .await;
    if let Ok(query) = &res
        && let Some(data) = &query.data
        && let Some(category) = &data.category
        && let Some(products) = &category.products
    {
        all_categorised_products.append(
            &mut products
                .edges
                .iter()
                .map(|p| {
                    Arc::new(CategorisedProduct {
                        product: p.node.clone(),
                        category_id: main_category.0.id.clone(),
                    })
                })
                .collect::<Vec<_>>(),
        );
        //Keep fetching next page
        debug!("fetched first products, eg: {:?}", products.edges.first());
        let mut next_cursor = products.page_info.end_cursor.clone();
        while let Some(cursor) = &mut next_cursor {
            let res = surf::post(saleor_api_url)
                .header("authorization-bearer", token)
                .run_graphql(GetCategoryProductsNext::build(
                    GetCategoryProductsNextVariables {
                        id: &main_category.0.id,
                        after: cursor,
                        channel,
                    },
                ))
                .await;
            if let Ok(query) = &res
                && let Some(data) = &query.data
                && let Some(category) = &data.category
                && let Some(products) = &category.products
            {
                all_categorised_products.append(
                    &mut products
                        .edges
                        .iter()
                        .map(|p| {
                            Arc::new(CategorisedProduct {
                                product: p.node.clone(),
                                category_id: main_category.0.id.clone(),
                            })
                        })
                        .collect::<Vec<_>>(),
                );
                debug!("fetched next products, eg: {:?}", products.edges.first());
                next_cursor.clone_from(&products.page_info.end_cursor);
            } else {
                error!("Failed fetching initial products! {:?}", &res);
                anyhow::bail!("Failed fetching initial products! {:?}", res);
            }
        }
    }
    info!("All products collected...");
    Ok(all_categorised_products)
}
