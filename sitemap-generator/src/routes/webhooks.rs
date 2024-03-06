use std::{fs::File, io::Write};

use anyhow::Context;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
};
use chrono::TimeZone;
use fd_lock::RwLock;
use flate2::{write::GzEncoder, Compression};
use saleor_app_sdk::{
    headers::SALEOR_API_URL_HEADER,
    webhooks::{
        utils::{get_webhook_event_type, EitherWebhookType},
        AsyncWebhookEventType,
    },
};
use sitemap_rs::{
    url::{ChangeFrequency, Url},
    url_set::UrlSet,
};
use tinytemplate::TinyTemplate;
use tokio::spawn;
use tracing::{debug, error, info};

use crate::{
    app::{AppError, AppState, XmlData, XmlDataType},
    queries::event_subjects_updated::{
        Category, CategoryUpdated, CollectionUpdated, PageUpdated, Product, ProductUpdated,
    },
};

pub async fn webhooks(
    headers: HeaderMap,
    State(state): State<AppState>,
    data: String,
) -> Result<StatusCode, AppError> {
    debug!("/api/webhooks");
    debug!("req: {:?}", data);
    debug!("headers: {:?}", headers);

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
                spawn(async move { update_sitemap_product(product, &url, state).await });
            }
            AsyncWebhookEventType::CategoryCreated
            | AsyncWebhookEventType::CategoryUpdated
            | AsyncWebhookEventType::CategoryDeleted => {
                let category: CategoryUpdated = serde_json::from_str(&data)?;
                spawn(async move { update_sitemap_category(category, &url, state).await });
            }
            AsyncWebhookEventType::PageCreated
            | AsyncWebhookEventType::PageUpdated
            | AsyncWebhookEventType::PageDeleted => {
                let page: PageUpdated = serde_json::from_str(&data)?;
                spawn(async move { update_sitemap_page(page, &url, state).await });
            }
            AsyncWebhookEventType::CollectionCreated
            | AsyncWebhookEventType::CollectionUpdated
            | AsyncWebhookEventType::CollectionDeleted => {
                let collection: CollectionUpdated = serde_json::from_str(&data)?;
                spawn(async move { update_sitemap_collection(collection, &url, state).await });
            }

            _ => (),
        },
        _ => (),
    }

    info!("got webhooks!");
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
        // TODO: when there are no keys, this will error. Work around that
        let mut xml_data = state.xml_cache.get_all(saleor_api_url).await?;
        let mut new_data = vec![];
        for x in xml_data.iter_mut() {
            if x.id == product.id && x.data_type == XmlDataType::Product {
                debug!(
                    "changed product {} found in xml_data, updating...",
                    product.slug
                );
                x.slug = product.slug.clone();
                x.relations = match &product.category {
                    Some(c) => vec![c.id.clone()],
                    None => vec![],
                };
            } else {
                debug!(
                    "changed product {} not found in xml_data, adding...",
                    product.slug
                );
                new_data.push(XmlData {
                    relations: match &product.category {
                        Some(c) => vec![c.id.clone()],
                        None => vec![],
                    },
                    id: product.id.clone(),
                    data_type: XmlDataType::Product,
                    slug: product.slug.clone(),
                })
            };
        }
        xml_data.append(&mut new_data);
        debug!("new xml_data : {:?}", &xml_data);
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
                        category: match x.relations.is_empty() {
                            false => {
                                let data = xml_data
                                    .iter()
                                    .find(|d| x.relations.iter().find(|r| **r == d.id).is_some());
                                match data {
                                    Some(d) => Some(Category {
                                        slug: d.slug.clone(),
                                        id: d.id.clone(),
                                    }),
                                    None => Some(Category {
                                        slug: "unknown".to_owned(),
                                        id: cynic::Id::new("unknown".to_owned()),
                                    }),
                                }
                            }
                            true => Some(Category {
                                slug: "unknown".to_owned(),
                                id: cynic::Id::new("unknown".to_owned()),
                            }),
                        },
                    }),
                };
                urls.push(tt.render("product_url", &context)?);
            }
        }
        debug!("new urls:{:?}", &urls);

        write_xml(
            urls,
            RwLock::new(
                File::options()
                    .create(true)
                    .write(true)
                    .open("./sitemap.xml")?,
            ),
        )
        .await?;
    } else {
        error!("Failed to update product, e: {:?}", product);
        anyhow::bail!("product not present in body");
    }
    debug!("Sitemap updated");
    Ok(())
}

async fn update_sitemap_category(
    category: CategoryUpdated,
    saleor_api_url: &str,
    state: AppState,
) -> anyhow::Result<()> {
    todo!()
}
async fn update_sitemap_collection(
    collection: CollectionUpdated,
    saleor_api_url: &str,
    state: AppState,
) -> anyhow::Result<()> {
    todo!()
}
async fn update_sitemap_page(
    page: PageUpdated,
    saleor_api_url: &str,
    state: AppState,
) -> anyhow::Result<()> {
    todo!()
}

async fn write_xml(urls: Vec<String>, mut file: RwLock<File>) -> anyhow::Result<()> {
    let mut f = file.write()?;
    let mut sitemap_urls: Vec<Url> = vec![];
    for url in urls {
        sitemap_urls.push(
            Url::builder(url)
                .change_frequency(ChangeFrequency::Weekly)
                .last_modified(chrono::offset::Utc::now().fixed_offset())
                .build()?,
        );
    }
    let url_set: UrlSet = UrlSet::new(sitemap_urls)?;
    debug!("Writing xml into file");
    f.set_len(0)?;
    let mut buf = Vec::<u8>::new();
    url_set.write(&mut buf)?;
    f.write_all(&buf)?;
    //let mut gzip = GzEncoder::new(f, Compression::default());
    todo!()
}
