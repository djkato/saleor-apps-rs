use cynic::{http::SurfExt, QueryBuilder};
use tracing::{debug, error, info};

use crate::{
    app::AppState,
    queries::{
        event_subjects_updated::{
            CategoryCreated, CollectionCreated, Page, PageCreated, ProductCreated,
        },
        get_all_categories::{
            Category3, GetCategoriesInitial, GetCategoriesNext, GetCategoriesNextVariables,
        },
        get_all_collections::{
            Collection, GetCollectionsInitial, GetCollectionsNext, GetCollectionsNextVariables,
        },
        get_all_pages::{self, GetPagesInitial, GetPagesNext, GetPagesNextVariables},
        get_all_products::{
            GetProductsInitial, GetProductsInitialVariables, GetProductsNext,
            GetProductsNextVariables, Product,
        },
    },
    sitemap::{
        event_handler::{write_db_to_file, write_url_set_to_file},
        ItemData, ItemType, Url, UrlSet,
    },
};

pub async fn regenerate(state: AppState, saleor_api_url: String) -> anyhow::Result<()> {
    info!("regeneration: fetching all categories, products, collections, pages");
    let app = state.saleor_app.lock().await;
    let auth_data = app.apl.get(&saleor_api_url).await?;

    let pages = get_all_pages(&saleor_api_url, &auth_data.token).await?;
    let collections = get_all_collections(&saleor_api_url, &auth_data.token).await?;
    let categories = get_all_categories(&saleor_api_url, &auth_data.token).await?;
    let products =
        get_all_products(&saleor_api_url, &state.target_channel, &auth_data.token).await?;
    info!(
        "regeneration: found {} products, {} categories, {} pages, {} collections",
        products.len(),
        categories.len(),
        pages.len(),
        collections.len()
    );
    info!("regeneration: creating sitemap data");
    let mut url_set = UrlSet::new();

    url_set.urls.append(
        &mut pages
            .into_iter()
            .filter_map(|p| {
                match Url::new(
                    PageCreated {
                        page: Some(Page {
                            id: p.id.clone(),
                            slug: p.slug.clone(),
                        }),
                    },
                    &state.sitemap_config,
                    ItemData {
                        id: p.id.inner().to_owned(),
                        slug: p.slug.clone(),
                        typ: ItemType::Page,
                    },
                    None,
                ) {
                    Ok(u) => Some(u),
                    Err(e) => {
                        error!("Error creating Url from page {:?}, {:?}", &p, e);
                        None
                    }
                }
            })
            .collect::<Vec<_>>(),
    );

    url_set.urls.append(
        &mut collections
            .into_iter()
            .filter_map(|p| {
                match Url::new(
                    CollectionCreated {
                        collection: Some(crate::queries::event_subjects_updated::Collection {
                            id: p.id.clone(),
                            slug: p.slug.clone(),
                        }),
                    },
                    &state.sitemap_config,
                    ItemData {
                        id: p.id.inner().to_owned(),
                        slug: p.slug.clone(),
                        typ: ItemType::Collection,
                    },
                    None,
                ) {
                    Ok(u) => Some(u),
                    Err(e) => {
                        error!("Error creating Url from collection {:?}, {:?}", &p, e);
                        None
                    }
                }
            })
            .collect::<Vec<_>>(),
    );

    url_set.urls.append(
        &mut categories
            .into_iter()
            .filter_map(|p| {
                match Url::new(
                    CategoryCreated {
                        category: Some(crate::queries::event_subjects_updated::Category2 {
                            id: p.id.clone(),
                            slug: p.slug.clone(),
                        }),
                    },
                    &state.sitemap_config,
                    ItemData {
                        id: p.id.inner().to_owned(),
                        slug: p.slug.clone(),
                        typ: ItemType::Category,
                    },
                    None,
                ) {
                    Ok(u) => Some(u),
                    Err(e) => {
                        error!("Error creating Url from category {:?}, {:?}", &p, e);
                        None
                    }
                }
            })
            .collect::<Vec<_>>(),
    );

    url_set.urls.append(
        &mut products
            .into_iter()
            .filter_map(|p| {
                match Url::new(
                    ProductCreated {
                        product: Some(crate::queries::event_subjects_updated::Product {
                            id: p.id.clone(),
                            slug: p.slug.clone(),
                            category: p.category.clone().map(|c| {
                                crate::queries::event_subjects_updated::Category {
                                    slug: c.slug,
                                    id: c.id,
                                }
                            }),
                        }),
                    },
                    &state.sitemap_config,
                    ItemData {
                        id: p.id.inner().to_owned(),
                        slug: p.slug.clone(),
                        typ: ItemType::Product,
                    },
                    p.category.clone().map(|c| ItemData {
                        id: c.id.inner().to_owned(),
                        slug: c.slug,
                        typ: ItemType::Category,
                    }),
                ) {
                    Ok(u) => Some(u),
                    Err(e) => {
                        error!("Error creating Url from product{:?}, {:?}", &p, e);
                        None
                    }
                }
            })
            .collect::<Vec<_>>(),
    );

    info!("regeneration: creating sitemap file");
    write_db_to_file(&url_set, &state.sitemap_config.target_folder).await?;
    write_url_set_to_file(&url_set, &state.sitemap_config.target_folder).await?;
    debug!("Wrote all files to disk");
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
) -> anyhow::Result<Vec<Product>> {
    debug!("Collecting all products...");
    let operation = GetProductsInitial::build(GetProductsInitialVariables { channel });
    let mut all_categorised_products: Vec<Product> = vec![];
    let res = surf::post(saleor_api_url)
        .header("authorization-bearer", token)
        .run_graphql(operation)
        .await;
    if let Ok(query) = &res
        && let Some(data) = &query.data
        && let Some(products) = &data.products
    {
        all_categorised_products.append(
            &mut products
                .edges
                .clone()
                .into_iter()
                .map(|p| p.node)
                .collect::<Vec<_>>(),
        );
        //Keep fetching next page
        debug!("fetched first products, eg: {:?}", products.edges.first());
        let mut next_cursor = products.page_info.end_cursor.clone();
        while let Some(cursor) = &mut next_cursor {
            let res = surf::post(saleor_api_url)
                .header("authorization-bearer", token)
                .run_graphql(GetProductsNext::build(GetProductsNextVariables {
                    after: cursor,
                    channel,
                }))
                .await;
            if let Ok(query) = &res
                && let Some(data) = &query.data
                && let Some(products) = &data.products
            {
                all_categorised_products.append(
                    &mut products
                        .edges
                        .clone()
                        .into_iter()
                        .map(|p| p.node)
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
