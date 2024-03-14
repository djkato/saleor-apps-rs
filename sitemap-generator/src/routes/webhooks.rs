use tokio::{fs::File, io::AsyncWriteExt};

use anyhow::Context;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
};
use chrono::{DateTime, Utc};
use saleor_app_sdk::{
    headers::SALEOR_API_URL_HEADER,
    webhooks::{
        utils::{get_webhook_event_type, EitherWebhookType},
        AsyncWebhookEventType,
    },
};
use sitemap_rs::{
    sitemap::Sitemap,
    sitemap_index::SitemapIndex,
    url::{Url},
    url_set::UrlSet,
};
use tinytemplate::TinyTemplate;
use tokio::spawn;
use tracing::{debug, error, info};

use crate::{
    app::{AppError, AppState, XmlData, XmlDataType},
    queries::event_subjects_updated::{
        Category, Category2, CategoryUpdated, Collection, CollectionUpdated, Page,
        PageUpdated, Product, ProductUpdated,
    },
};

pub async fn webhooks(
    headers: HeaderMap,
    State(state): State<AppState>,
    data: String,
) -> Result<StatusCode, AppError> {
    debug!("/api/webhooks");
    //debug!("req: {:?}", data);
    //debug!("headers: {:?}", headers);

    let url = headers
        .get(SALEOR_API_URL_HEADER)
        .context("missing saleor api url header")?
        .to_str()?
        .to_owned();
    let event_type = get_webhook_event_type(&headers)?;
    match event_type {
        EitherWebhookType::Async(a) => match a {
            AsyncWebhookEventType::ProductUpdated
            | AsyncWebhookEventType::ProductCreated
            | AsyncWebhookEventType::ProductDeleted => {
                let product: ProductUpdated = serde_json::from_str(&data)?;
                spawn(async move {
                    if let Err(e) = update_sitemap_product(product, &url, state).await {
                        error!("Error processing Product, e: {:?}", e);
                    }
                });
            }
            AsyncWebhookEventType::CategoryCreated
            | AsyncWebhookEventType::CategoryUpdated
            | AsyncWebhookEventType::CategoryDeleted => {
                let category: CategoryUpdated = serde_json::from_str(&data)?;
                spawn(async move {
                    if let Err(e) = update_sitemap_category(category, &url, state).await {
                        error!("Error processing Category, e: {:?}", e);
                    }
                });
            }
            AsyncWebhookEventType::PageCreated
            | AsyncWebhookEventType::PageUpdated
            | AsyncWebhookEventType::PageDeleted => {
                let page: PageUpdated = serde_json::from_str(&data)?;
                spawn(async move {
                    if let Err(e) = update_sitemap_page(page, &url, state).await {
                        error!("Error processing Page, e: {:?}", e);
                    }
                });
            }
            AsyncWebhookEventType::CollectionCreated
            | AsyncWebhookEventType::CollectionUpdated
            | AsyncWebhookEventType::CollectionDeleted => {
                let collection: CollectionUpdated = serde_json::from_str(&data)?;
                spawn(async move {
                    if let Err(e) = update_sitemap_collection(collection, &url, state).await {
                        error!("Error processing Collection, e: {:?}", e);
                    }
                });
            }

            _ => (),
        },
        _ => (),
    }

    info!("webhook proccessed");
    Ok(StatusCode::OK)
}

async fn update_sitemap_product(
    product: ProductUpdated,
    saleor_api_url: &str,
    state: AppState,
) -> anyhow::Result<()> {
    debug!("Product got changed!, {:?}", &product);
    if let Some(product) = product.product {
        // Update or add the product
        let xml_cache = state.xml_cache.lock().await;
        let mut xml_data = match xml_cache.get_all(saleor_api_url).await {
            Ok(d) => d,
            Err(e) => {
                error!("Error, {:?}. no xml cache present?", e);
                vec![]
            }
        };

        //find the product in xml data and update / create it
        let mut new_data = vec![];
        let cloned_xml_data = xml_data.clone();
        //debug!("{:?}", xml_data);
        match xml_data
            .iter_mut()
            .find(|x| x.id == product.id && x.data_type == XmlDataType::Product)
        {
            Some(x) => {
                //Check if the slug or category.slug has changed, else ignore the change and continue
                debug!("{} == {}", x.slug, product.slug);
                if x.slug == product.slug {
                    match &product.category {
                        Some(c) => {
                            if let Some(xml_c) = cloned_xml_data
                                .iter()
                                .find(|d| d.id == c.id && d.data_type == XmlDataType::Category)
                            {
                                if xml_c.slug == c.slug {
                                    debug!("Products url didn't change, skipping...");
                                    return Ok(());
                                }
                            }
                        }
                        None => {
                            debug!("Products url didn't change, skipping...");
                            return Ok(());
                        }
                    }
                }
                debug!(
                    "changed product {} found in xml_data, updating...",
                    product.slug
                );
                x.slug.clone_from(&product.slug);
                x.relations = match &product.category {
                    Some(c) => vec![c.id.clone()],
                    None => vec![],
                };
                x.last_modified = chrono::offset::Utc::now().fixed_offset();
            }
            None => {
                debug!(
                    "changed product {} not found in xml_data, adding...",
                    product.slug
                );
                new_data.push(XmlData {
                    last_modified: chrono::offset::Utc::now().fixed_offset(),
                    relations: match &product.category {
                        Some(c) => {
                            vec![c.id.clone()]
                        }
                        None => vec![],
                    },
                    id: product.id.clone(),
                    data_type: XmlDataType::Product,
                    slug: product.slug.clone(),
                });
            }
        };
        //See if produts category exists
        if let Some(c) = &product.category {
            if let Some(xml_cat) = xml_data
                .iter_mut()
                .find(|x| x.id == c.id && x.data_type == XmlDataType::Category)
            {
                xml_cat.slug.clone_from(&c.slug);
                xml_cat.last_modified = chrono::offset::Utc::now().fixed_offset();
                // If the category exists but product isn't in relation to it yet,
                // add it
                if !xml_cat
                    .relations
                    .iter().any(|c| *c == product.id)
                {
                    xml_cat.relations.push(product.id.clone());
                }
            //if cat isn't in xml data, add it
            } else {
                new_data.push(XmlData {
                    last_modified: chrono::offset::Utc::now().fixed_offset(),
                    id: c.id.clone(),
                    slug: c.slug.clone(),
                    data_type: XmlDataType::Category,
                    relations: vec![product.id.clone()],
                })
            }
        }
        xml_data.append(&mut new_data);
        //create urls
        let mut urls = vec![];
        for x in xml_data.iter() {
            if x.data_type == XmlDataType::Product {
                let mut tt = TinyTemplate::new();
                tt.add_template("product_url", &state.sitemap_config.product_template)?;
                let context = ProductUpdated {
                    product: Some(Product {
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
                            Some(c) => Some(Category {
                                slug: c.slug.clone(),
                                id: c.id.clone(),
                            }),
                            None => Some(Category {
                                slug: "unknown".to_owned(),
                                id: cynic::Id::new("unknown".to_owned()),
                            }),
                        },
                    }),
                };
                urls.push(
                    Url::builder(tt.render("product_url", &context)?)
                        .last_modified(x.last_modified)
                        .build()?,
                );
            }
        }
        //debug!("new urls:{:?}", &urls);

        write_xml(urls, &state, XmlDataType::Product).await?;
        xml_cache.set(xml_data, saleor_api_url).await?;
    } else {
        error!("Failed to update product, e: {:?}", product);
        anyhow::bail!("product not present in in webhook");
    }
    info!("Sitemap updated, cause: product");
    Ok(())
}

async fn update_sitemap_category(
    category: CategoryUpdated,
    saleor_api_url: &str,
    state: AppState,
) -> anyhow::Result<()> {
    if let Some(category) = category.category {
        let xml_cache = state.xml_cache.lock().await;
        let mut xml_data = xml_cache.get_all(saleor_api_url).await?;
        let mut affected_product_ids = vec![];
        let mut new_xml_data = vec![];
        //check if template of product includes categories in url
        let is_category_in_product_url = state.sitemap_config.product_template.contains("category");
        match xml_data
            .iter_mut()
            .find(|c| c.id == category.id && c.data_type == XmlDataType::Category)
        {
            Some(xml_c) => {
                // if it changed, update
                if xml_c.slug == category.slug {
                    debug!("Category url didn't change, skipping...");
                    return Ok(());
                }
                debug!("Category url changed, updating...");
                xml_c.slug.clone_from(&category.slug);
                xml_c.last_modified = chrono::offset::Utc::now().fixed_offset();
                if is_category_in_product_url {
                    debug!("{} products affected by change", affected_product_ids.len());
                    affected_product_ids.append(&mut xml_c.relations.clone());
                }
            }
            None => {
                //Add category if it doesn't exist
                debug!("Category not found in cache, adding...");
                new_xml_data.push(XmlData {
                    relations: vec![],
                    last_modified: chrono::offset::Utc::now().fixed_offset(),
                    data_type: XmlDataType::Category,
                    slug: category.slug.clone(),
                    id: category.id.clone(),
                })
            }
        }
        //update affected products' last_modified
        if is_category_in_product_url {
            for prod_id in affected_product_ids {
                if let Some(xml_prod) = xml_data
                    .iter_mut()
                    .find(|p| p.id == prod_id && p.data_type == XmlDataType::Product)
                {
                    match xml_prod.relations.iter().find(|c| *c == &category.id) {
                        Some(_) => {
                            xml_prod.last_modified = chrono::offset::Utc::now().fixed_offset();
                        }
                        None => {
                            debug!("product in categories relation doesn't have the same relation back, what happened? Fixing...");
                            xml_prod.relations = vec![category.id.clone()];
                            xml_prod.last_modified = chrono::offset::Utc::now().fixed_offset();
                        }
                    };
                }
            }
        }

        xml_data.append(&mut new_xml_data);
        let mut category_urls = vec![];
        let mut product_urls = vec![];
        //Create urls
        for x in xml_data.iter() {
            let mut tt = TinyTemplate::new();
            if is_category_in_product_url && x.data_type == XmlDataType::Product {
                tt.add_template("product_url", &state.sitemap_config.product_template)?;
                let context;
                //If current xml products category is this changed category, just use that instead
                //of searching for it again
                match x.relations.iter().find(|c| *c == &category.id) {
                    Some(_) => {
                        context = ProductUpdated {
                            product: Some(Product {
                                id: x.id.clone(),
                                slug: x.slug.clone(),
                                category: Some(Category {
                                    slug: category.slug.clone(),
                                    id: category.id.clone(),
                                }),
                            }),
                        };
                    }
                    None => {
                        context = ProductUpdated {
                            product: Some(Product {
                                id: x.id.clone(),
                                slug: x.slug.clone(),
                                category: match xml_data.iter().find(|all| {
                                    x.relations
                                        .iter()
                                        .find(|rel| {
                                            all.id == **rel
                                                && all.data_type == XmlDataType::Category
                                        })
                                        .is_some()
                                }) {
                                    Some(c) => Some(Category {
                                        slug: c.slug.clone(),
                                        id: c.id.clone(),
                                    }),
                                    None => Some(Category {
                                        slug: "unknown".to_owned(),
                                        id: cynic::Id::new("unknown".to_owned()),
                                    }),
                                },
                            }),
                        };
                    }
                }
                product_urls.push(
                    Url::builder(tt.render("product_url", &context)?)
                        .last_modified(x.last_modified)
                        .build()?,
                );
            }
            if x.data_type == XmlDataType::Category {
                tt.add_template("category_url", &state.sitemap_config.category_template)?;
                let context = CategoryUpdated {
                    category: Some(Category2 {
                        id: x.id.clone(),
                        slug: x.slug.clone(),
                    }),
                };
                category_urls.push(
                    Url::builder(tt.render("category_url", &context)?)
                        .last_modified(x.last_modified)
                        .build()?,
                );
            }
        }
        //and write
        if is_category_in_product_url {
            write_xml(product_urls, &state, XmlDataType::Product).await?;
        }
        write_xml(category_urls, &state, XmlDataType::Category).await?;
        xml_cache.set(xml_data, saleor_api_url).await?;
    } else {
        error!("Failed to update category, e:{:?}", category);
        anyhow::bail!("Category not present in webhook");
    }
    info!("Sitemap updated, cause: category");
    Ok(())
}
async fn update_sitemap_collection(
    collection: CollectionUpdated,
    saleor_api_url: &str,
    state: AppState,
) -> anyhow::Result<()> {
    if let Some(collection) = collection.collection {
        let xml_cache = state.xml_cache.lock().await;
        let mut xml_data = xml_cache.get_all(saleor_api_url).await?;
        let mut new_xml_data = vec![];

        match xml_data
            .iter_mut()
            .find(|c| c.id == collection.id && c.data_type == XmlDataType::Collection)
        {
            Some(xml_col) => {
                if xml_col.slug == collection.slug {
                    debug!("Collection url didn't change, skipping");
                    return Ok(());
                }
                xml_col.slug = collection.slug;
                xml_col.last_modified = chrono::offset::Utc::now().fixed_offset();
            }
            None => {
                debug!("Collection not cached, adding...");
                new_xml_data.push(XmlData {
                    slug: collection.slug,
                    id: collection.id,
                    last_modified: chrono::offset::Utc::now().fixed_offset(),
                    relations: vec![],
                    data_type: XmlDataType::Collection,
                })
            }
        }

        xml_data.append(&mut new_xml_data);

        //create urls
        let mut collection_urls = vec![];
        for xml_col in xml_data.iter() {
            if xml_col.data_type == XmlDataType::Collection {
                let mut tt = TinyTemplate::new();
                tt.add_template("collection_url", &state.sitemap_config.collection_template)?;
                let context = CollectionUpdated {
                    collection: Some(Collection {
                        slug: xml_col.slug.clone(),
                        id: xml_col.id.clone(),
                    }),
                };
                collection_urls.push(
                    Url::builder(tt.render("collection_url", &context)?)
                        .last_modified(xml_col.last_modified)
                        .build()?,
                );
            }
        }
        write_xml(collection_urls, &state, XmlDataType::Collection).await?;
        xml_cache.set(xml_data, saleor_api_url).await?;
    } else {
        error!("Failed to update collection, e:{:?}", collection);
        anyhow::bail!("Collection not present in webhook");
    }

    info!("Sitemap updated, cause: collection");
    Ok(())
}
async fn update_sitemap_page(
    page: PageUpdated,
    saleor_api_url: &str,
    state: AppState,
) -> anyhow::Result<()> {
    if let Some(page) = page.page {
        let xml_cache = state.xml_cache.lock().await;
        let mut xml_data = xml_cache.get_all(saleor_api_url).await?;
        let mut new_xml_data = vec![];

        match xml_data
            .iter_mut()
            .find(|p| p.id == page.id && p.data_type == XmlDataType::Page)
        {
            Some(xml_page) => {
                if xml_page.slug == page.slug {
                    debug!("Page url didn't change, skipping");
                    return Ok(());
                }
                xml_page.slug = page.slug;
                xml_page.last_modified = chrono::offset::Utc::now().fixed_offset();
            }
            None => {
                debug!("Page not cached, adding...");
                new_xml_data.push(XmlData {
                    slug: page.slug,
                    id: page.id,
                    last_modified: chrono::offset::Utc::now().fixed_offset(),
                    relations: vec![],
                    data_type: XmlDataType::Page,
                })
            }
        }

        xml_data.append(&mut new_xml_data);
        //create urls
        let mut page_urls = vec![];
        for xml_page in xml_data.iter() {
            if xml_page.data_type == XmlDataType::Page {
                let mut tt = TinyTemplate::new();
                tt.add_template("page_url", &state.sitemap_config.pages_template)?;
                let context = PageUpdated {
                    page: Some(Page {
                        slug: xml_page.slug.clone(),
                        id: xml_page.id.clone(),
                    }),
                };
                page_urls.push(
                    Url::builder(tt.render("page_url", &context)?)
                        .last_modified(xml_page.last_modified)
                        .build()?,
                );
            }
        }
        write_xml(page_urls, &state, XmlDataType::Page).await?;
        xml_cache.set(xml_data, saleor_api_url).await?;
    } else {
        error!("Failed to update Page, e:{:?}", page);
        anyhow::bail!("Page not present in webhook");
    }

    info!("Sitemap updated, cause: Page");
    Ok(())
}

pub async fn write_xml(
    urls: Vec<Url>,
    state: &AppState,
    type_group: XmlDataType,
) -> anyhow::Result<()> {
    //Acquire lock first, so only one write_xml function can start computing
    let mut f = File::options()
        .create(true)
        .write(true)
        .open(format!(
            "{}/sitemap-{:?}-0.xml",
            state.sitemap_config.target_folder, type_group
        ))
        .await?;
    let mut sitemap_urls: Vec<Url> = vec![];
    for url in urls.clone() {
        sitemap_urls.push(url);
    }
    let url_set: UrlSet = UrlSet::new(sitemap_urls)?;
    debug!("Writing xml into file");

    //f.set_len(0)?;
    let mut buf = Vec::<u8>::new();
    url_set.write(&mut buf)?;
    //TODO: Gzip the buffer before testing size. Size limit per sitemap should be ~= 10mb

    //now check if buffer's over limit, else slice em up into multiple sitemaps
    let len = buf.len() * std::mem::size_of::<u8>();
    if len > 200000 {
        let file_amount = (len as f32 / 150000_f32).ceil() as usize;
        let sliced_urls: Vec<&[Url]> = urls.chunks(file_amount).collect();

        let mut sitemaps: Vec<UrlSet> = vec![];
        for urls in sliced_urls {
            for url in urls.iter().cloned() {
                let mut sitemap_urls: Vec<Url> = vec![];
                sitemap_urls.push(url);
                sitemaps.push(UrlSet::new(sitemap_urls)?);
            }
        }

        for (i, sitemap) in sitemaps.into_iter().enumerate() {
            let mut new_buf = Vec::<u8>::new();
            sitemap.write(&mut new_buf)?;
            let len = new_buf.len() * std::mem::size_of::<u8>();
            if len > 200000 {
                error!("Sitemap is too big even after splitting. Gosh I wish I was better at math")
            }
            let mut f = File::options()
                .create(true)
                .write(true)
                .open(format!(
                    "{}/sitemap-{:?}-{i}.xml",
                    state.sitemap_config.target_folder, type_group
                ))
                .await?;
            f.write_all(&new_buf).await?;
        }
    } else {
        f.write_all(&buf).await?;
    }
    //let mut gzip = GzEncoder::new(f, Compression::default());
    update_sitemap_index(state).await?;
    Ok(())
}

async fn update_sitemap_index(state: &AppState) -> anyhow::Result<()> {
    use std::fs::read_dir;
    let dir = read_dir(&state.sitemap_config.target_folder)?;
    let paths = dir
        .filter_map(|f| f.ok())
        .map(|e| e.path())
        .filter_map(|path| {
            if path
                .extension()
                .map_or(false, |ext| ext == "xml" || ext == "gz")
                && !path.to_string_lossy().to_string().contains("sitemap-index")
            {
                Some(path)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let sitemaps: Vec<Sitemap> = paths
        .into_iter()
        .map(|p| {
            Sitemap::new(
                format!(
                    "{}/{}",
                    state.sitemap_config.index_hostname,
                    p.file_name()
                        .expect("file dissapeared or broke during sitemap-index construction")
                        .to_string_lossy()
                ),
                p.metadata().map_or(None, |meta| {
                    meta.modified().map_or(None, |modified| {
                        let dt_utc: DateTime<Utc> = modified.into();
                        Some(dt_utc.fixed_offset())
                    })
                }),
            )
        })
        .collect::<Vec<_>>();
    let sitemap_index = SitemapIndex::new(sitemaps)?;
    let mut file = File::options()
        .create(true)
        .write(true)
        .open(format!(
            "{}/sitemap-index.xml",
            state.sitemap_config.target_folder
        ))
        .await?;

    let mut buf = Vec::<u8>::new();
    sitemap_index.write(&mut buf)?;
    file.write_all(&mut buf).await?;

    Ok(())
}
