use std::{str::FromStr, sync::Arc};

use anyhow::Context;
use axum::{
    extract::Json,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use cynic::{http::SurfExt, QueryBuilder};
use saleor_app_sdk::{AuthData, AuthToken};
use sitemap_rs::url::Url;
use tinytemplate::TinyTemplate;
use tokio::spawn;
use tracing::{debug, error, info, trace};

use crate::{
    app::{AppError, AppState, XmlData, XmlDataType},
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
    routes::webhooks::write_xml,
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
    let xml_cache = state.xml_cache.lock().await;
    let app = state.saleor_app.lock().await;
    let auth_data = app.apl.get(&saleor_api_url).await?;

    let mut categories: Vec<(Category3, Vec<Arc<CategorisedProduct>>)> =
        get_all_categories(&saleor_api_url, &auth_data.token)
            .await?
            .into_iter()
            .map(|c| (c, vec![]))
            .collect();
    let mut products = vec![];
    for category in categories.iter_mut() {
        products.append(&mut get_all_products(&saleor_api_url, &auth_data.token, category).await?);
    }
    let pages = get_all_pages(&saleor_api_url, &auth_data.token).await?;
    let collections = get_all_collections(&saleor_api_url, &auth_data.token).await?;
    info!(
        "regeneration: found {} products, {} categories, {} pages, {} collections",
        products.len(),
        categories.len(),
        pages.len(),
        collections.len()
    );
    info!("regeneration: creating xml data and caching it");
    let mut xml_data = vec![];
    xml_data.append(
        &mut categories
            .into_iter()
            .map(|c| XmlData {
                slug: c.0.slug,
                last_modified: chrono::DateTime::<chrono::Utc>::from_str(&c.0.updated_at.0)
                    .map_or(chrono::offset::Utc::now().fixed_offset(), |d| {
                        d.fixed_offset()
                    }),
                id: c.0.id,
                relations: c.1.iter().map(|p| p.product.id.clone()).collect::<Vec<_>>(),
                data_type: XmlDataType::Category,
            })
            .collect::<Vec<_>>(),
    );
    xml_data.append(
        &mut products
            .into_iter()
            .map(|p| XmlData {
                data_type: XmlDataType::Product,
                relations: vec![p.category_id.clone()],
                id: p.product.id.clone(),
                last_modified: chrono::DateTime::<chrono::Utc>::from_str(&p.product.updated_at.0)
                    .map_or(chrono::offset::Utc::now().fixed_offset(), |d| {
                        d.fixed_offset()
                    }),
                slug: p.product.slug.clone(),
            })
            .collect(),
    );
    xml_data.append(
        &mut pages
            .into_iter()
            .map(|p| XmlData {
                data_type: XmlDataType::Page,
                relations: vec![],
                id: p.id.clone(),
                last_modified: match p.published_at {
                    Some(d) => chrono::DateTime::<chrono::Utc>::from_str(&d.0)
                        .map_or(chrono::offset::Utc::now().fixed_offset(), |d| {
                            d.fixed_offset()
                        }),
                    None => chrono::offset::Utc::now().fixed_offset(),
                },
                slug: p.slug.clone(),
            })
            .collect(),
    );
    xml_data.append(
        &mut collections
            .into_iter()
            .map(|c| XmlData {
                slug: c.slug,
                last_modified: chrono::offset::Utc::now().fixed_offset(),
                id: c.id,
                relations: vec![],
                data_type: XmlDataType::Category,
            })
            .collect::<Vec<_>>(),
    );

    xml_cache.set(xml_data.clone(), &saleor_api_url).await?;
    info!("regeneration: xml_cache was set");

    //create urls
    info!("regeneration: creating urls");
    let mut page_urls = vec![];
    let mut product_urls = vec![];
    let mut category_urls = vec![];
    let mut collection_urls = vec![];

    for x in xml_data.iter() {
        match x.data_type {
            XmlDataType::Page => {
                let mut tt = TinyTemplate::new();
                tt.add_template("page_url", &state.sitemap_config.pages_template)?;
                let context = PageUpdated {
                    page: Some(event_subjects_updated::Page {
                        slug: x.slug.clone(),
                        id: x.id.clone(),
                    }),
                };
                let page_url = Url::builder(tt.render("page_url", &context)?)
                    .last_modified(x.last_modified)
                    .build()?;
                trace!("Created Page url: {}", &page_url.location);
                page_urls.push(page_url);
            }
            XmlDataType::Product => {
                let mut tt = TinyTemplate::new();
                tt.add_template("product_url", &state.sitemap_config.product_template)?;
                let context = ProductUpdated {
                    product: Some(event_subjects_updated::Product {
                        id: x.id.clone(),
                        slug: x.slug.clone(),
                        category: match xml_data.iter().find(|all| {
                            x.relations
                                .iter()
                                .find(|rel| {
                                    all.id == **rel && all.data_type == XmlDataType::Category
                                })
                                .is_some()
                        }) {
                            Some(c) => Some(event_subjects_updated::Category {
                                slug: c.slug.clone(),
                                id: c.id.clone(),
                            }),
                            None => Some(event_subjects_updated::Category {
                                slug: "unknown".to_owned(),
                                id: cynic::Id::new("unknown".to_owned()),
                            }),
                        },
                    }),
                };
                let product_url = Url::builder(tt.render("product_url", &context)?)
                    .last_modified(x.last_modified)
                    .build()?;

                trace!("Created Page url: {}", &product_url.location);
                product_urls.push(product_url);
            }
            XmlDataType::Category => {
                let mut tt = TinyTemplate::new();
                tt.add_template("category_url", &state.sitemap_config.category_template)?;
                let context = CategoryUpdated {
                    category: Some(event_subjects_updated::Category2 {
                        id: x.id.clone(),
                        slug: x.slug.clone(),
                    }),
                };
                let category_url = Url::builder(tt.render("category_url", &context)?)
                    .last_modified(x.last_modified)
                    .build()?;

                trace!("Created category url: {}", &category_url.location);
                category_urls.push(category_url);
            }
            XmlDataType::Collection => {
                let mut tt = TinyTemplate::new();
                tt.add_template("coll_url", &state.sitemap_config.collection_template)?;
                let context = CollectionUpdated {
                    collection: Some(event_subjects_updated::Collection {
                        slug: x.slug.clone(),
                        id: x.id.clone(),
                    }),
                };
                let collection_url = Url::builder(tt.render("coll_url", &context)?)
                    .last_modified(x.last_modified)
                    .build()?;

                trace!("Created collection url: {}", &collection_url.location);
                collection_urls.push(collection_url);
            }
        }
    }
    write_xml(page_urls, &state, XmlDataType::Page).await?;
    write_xml(collection_urls, &state, XmlDataType::Collection).await?;
    write_xml(category_urls, &state, XmlDataType::Category).await?;
    write_xml(product_urls, &state, XmlDataType::Product).await?;
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
        loop {
            if let Some(cursor) = &mut next_cursor {
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
                    if !pages.page_info.has_next_page {
                        break;
                    }
                    next_cursor.clone_from(&pages.page_info.end_cursor);
                } else {
                    error!("Failed fetching initial pages! {:?}", &res);
                    anyhow::bail!("Failed fetching initial pages! {:?}", res);
                }
            } else {
                break;
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
        loop {
            if let Some(cursor) = &mut next_cursor {
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
                        "fetched first categories, eg.:{:?}",
                        &categories.edges.first()
                    );
                    if !categories.page_info.has_next_page {
                        break;
                    }
                    next_cursor.clone_from(&categories.page_info.end_cursor);
                } else {
                    error!("Failed fetching initial pages! {:?}", &res);
                    anyhow::bail!("Failed fetching initial pages! {:?}", res);
                }
            } else {
                break;
            }
        }
    } else {
        error!("Failed fetching initial pages! {:?}", &res);
        anyhow::bail!("Failed fetching initial pages! {:?}", res);
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
        loop {
            if let Some(cursor) = &mut next_cursor {
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
                    if !collections.page_info.has_next_page {
                        break;
                    }
                    next_cursor.clone_from(&collections.page_info.end_cursor);
                } else {
                    error!("Failed fetching initial collecnios! {:?}", &res);
                    anyhow::bail!("Failed fetching initial collections! {:?}", res);
                }
            } else {
                break;
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
    token: &str,
    main_category: &mut (Category3, Vec<Arc<CategorisedProduct>>),
) -> anyhow::Result<Vec<Arc<CategorisedProduct>>> {
    debug!("Collecting all products...");
    let operation = GetCategoryProductsInitial::build(GetCategoryProductsInitialVariables {
        id: &main_category.0.id,
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
        loop {
            if let Some(cursor) = &mut next_cursor {
                let res = surf::post(saleor_api_url)
                    .header("authorization-bearer", token)
                    .run_graphql(GetCategoryProductsNext::build(
                        GetCategoryProductsNextVariables {
                            id: &main_category.0.id,
                            after: cursor,
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
                    if !products.page_info.has_next_page {
                        break;
                    }
                    next_cursor.clone_from(&products.page_info.end_cursor);
                } else {
                    error!("Failed fetching initial products! {:?}", &res);
                    anyhow::bail!("Failed fetching initial products! {:?}", res);
                }
            } else {
                break;
            }
        }
    } else {
        error!("Failed fetching initial products! {:?}", &res);
        anyhow::bail!("Failed fetching initial products! {:?}", res);
    };
    info!("All products collected...");
    Ok(all_categorised_products)
}
