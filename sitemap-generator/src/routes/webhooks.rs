use anyhow::Context;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
};
use saleor_app_sdk::{
    headers::SALEOR_API_URL_HEADER,
    webhooks::{
        utils::{get_webhook_event_type, EitherWebhookType},
        AsyncWebhookEventType,
    },
};
use tracing::{debug, info};

use crate::{
    app::{AppError, AppState},
    queries::event_subjects_updated::{
        CategoryCreated, CategoryDeleted, CategoryUpdated, CollectionCreated, CollectionDeleted,
        CollectionUpdated, PageCreated, PageDeleted, PageUpdated, ProductCreated, ProductDeleted,
        ProductUpdated,
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
    if let EitherWebhookType::Async(a) = event_type {
        // TODO: Extract this into a function so You can check what the error was if something fails
        match a {
            AsyncWebhookEventType::ProductUpdated => {
                let product: ProductUpdated = serde_json::from_str(&data)?;
            }
            AsyncWebhookEventType::ProductCreated => {
                let product: ProductCreated = serde_json::from_str(&data)?;
            }
            AsyncWebhookEventType::ProductDeleted => {
                let product: ProductDeleted = serde_json::from_str(&data)?;
            }
            AsyncWebhookEventType::CategoryCreated => {
                let category: CategoryCreated = serde_json::from_str(&data)?;
            }
            AsyncWebhookEventType::CategoryUpdated => {
                let category: CategoryUpdated = serde_json::from_str(&data)?;
            }
            AsyncWebhookEventType::CategoryDeleted => {
                let category: CategoryDeleted = serde_json::from_str(&data)?;
            }
            AsyncWebhookEventType::PageCreated => {
                let page: PageCreated = serde_json::from_str(&data)?;
            }
            AsyncWebhookEventType::PageUpdated => {
                let page: PageUpdated = serde_json::from_str(&data)?;
            }
            AsyncWebhookEventType::PageDeleted => {
                let page: PageDeleted = serde_json::from_str(&data)?;
            }
            AsyncWebhookEventType::CollectionCreated => {
                let collection: CollectionCreated = serde_json::from_str(&data)?;
            }
            AsyncWebhookEventType::CollectionUpdated => {
                let collection: CollectionUpdated = serde_json::from_str(&data)?;
            }
            AsyncWebhookEventType::CollectionDeleted => {
                let collection: CollectionDeleted = serde_json::from_str(&data)?;
            }
            _ => (),
        }
    }

    info!("webhook proccessed");
    Ok(StatusCode::OK)
}

// pub async fn write_xml(
//     urls: Vec<Url>,
//     state: &AppState,
//     type_group: XmlDataType,
// ) -> anyhow::Result<()> {
//     //Acquire lock first, so only one write_xml function can start computing
//     let mut f = File::options()
//         .create(true)
//         .write(true)
//         .open(format!(
//             "{}/sitemap-{:?}-0.xml",
//             state.sitemap_config.target_folder, type_group
//         ))
//         .await?;
//     let mut sitemap_urls: Vec<Url> = vec![];
//     for url in urls.clone() {
//         sitemap_urls.push(url);
//     }
//     let url_set: UrlSet = UrlSet::new(sitemap_urls)?;
//     debug!("Writing xml into file");
//
//     //f.set_len(0)?;
//     let mut buf = Vec::<u8>::new();
//     url_set.write(&mut buf)?;
//     //TODO: Gzip the buffer before testing size. Size limit per sitemap should be ~= 10mb
//
//     //now check if buffer's over limit, else slice em up into multiple sitemaps
//     let len = buf.len() * std::mem::size_of::<u8>();
//     if len > 200000 {
//         let file_amount = (len as f32 / 150000_f32).ceil() as usize;
//         let sliced_urls: Vec<&[Url]> = urls.chunks(file_amount).collect();
//
//         let mut sitemaps: Vec<UrlSet> = vec![];
//         for urls in sliced_urls {
//             for url in urls.iter().cloned() {
//                 let sitemap_urls = vec![url];
//                 sitemaps.push(UrlSet::new(sitemap_urls)?);
//             }
//         }
//
//         for (i, sitemap) in sitemaps.into_iter().enumerate() {
//             let mut new_buf = Vec::<u8>::new();
//             sitemap.write(&mut new_buf)?;
//             let len = new_buf.len() * std::mem::size_of::<u8>();
//             if len > 200000 {
//                 error!("Sitemap is too big even after splitting. Gosh I wish I was better at math")
//             }
//             let mut f = File::options()
//                 .create(true)
//                 .write(true)
//                 .open(format!(
//                     "{}/sitemap-{:?}-{i}.xml",
//                     state.sitemap_config.target_folder, type_group
//                 ))
//                 .await?;
//             f.write_all(&new_buf).await?;
//         }
//     } else {
//         f.write_all(&buf).await?;
//     }
//     //let mut gzip = GzEncoder::new(f, Compression::default());
//     update_sitemap_index(state).await?;
//     Ok(())
// }
//
// async fn update_sitemap_index(state: &AppState) -> anyhow::Result<()> {
//     use std::fs::read_dir;
//     let dir = read_dir(&state.sitemap_config.target_folder)?;
//     let paths = dir
//         .filter_map(|f| f.ok())
//         .map(|e| e.path())
//         .filter_map(|path| {
//             if path
//                 .extension()
//                 .map_or(false, |ext| ext == "xml" || ext == "gz")
//                 && !path.to_string_lossy().to_string().contains("sitemap_index")
//             {
//                 Some(path)
//             } else {
//                 None
//             }
//         })
//         .collect::<Vec<_>>();
//
//     let sitemaps: Vec<Sitemap> = paths
//         .into_iter()
//         .filter_map(|p| {
//             if let Some(file_name) = p.file_name() {
//                 Some(Sitemap::new(
//                     format!(
//                         "{}/{}",
//                         state.sitemap_config.index_hostname,
//                         file_name.to_string_lossy()
//                     ),
//                     p.metadata().map_or(None, |meta| {
//                         meta.modified().map_or(None, |modified| {
//                             let dt_utc: DateTime<Utc> = modified.into();
//                             Some(dt_utc.fixed_offset())
//                         })
//                     }),
//                 ))
//             } else {
//                 error!("file dissapeared or broke during sitemap_index construction");
//                 None
//             }
//         })
//         .collect::<Vec<_>>();
//     let sitemap_index = SitemapIndex::new(sitemaps)?;
//     let mut file = File::options()
//         .create(true)
//         .write(true)
//         .open(format!(
//             "{}/sitemap_index.xml",
//             state.sitemap_config.target_folder
//         ))
//         .await?;
//
//     let mut buf = Vec::<u8>::new();
//     sitemap_index.write(&mut buf)?;
//     file.write_all(&buf).await?;
//
//     Ok(())
// }
