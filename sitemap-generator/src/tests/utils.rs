use axum::{
    body::Body,
    http::{Request, Response, StatusCode},
    routing::RouterIntoService,
};
use rand::{
    distributions::{Distribution, Standard},
    seq::SliceRandom,
    Rng,
};
use saleor_app_sdk::{
    apl::AplType,
    config::Config,
    headers::{SALEOR_API_URL_HEADER, SALEOR_EVENT_HEADER},
    webhooks::{utils::EitherWebhookType, AsyncWebhookEventType},
};
use tower::{Service, ServiceExt};
use tracing::Level;

use crate::{
    app::{trace_to_std, SitemapConfig},
    queries::event_subjects_updated::{
        Category, Category2, CategoryUpdated, Collection, CollectionUpdated, Page, PageUpdated,
        Product, ProductUpdated,
    },
    sitemap::{ItemData, ItemType, Url},
};

pub fn init_tracing() {
    let config = Config {
        apl: AplType::File,
        apl_url: "redis://localhost:6379".to_string(),
        log_level: Level::TRACE,
        app_api_base_url: "http://localhost:3000".to_string(),
        app_iframe_base_url: "http://localhost:3000".to_string(),
        required_saleor_version: "^3.13".to_string(),
    };
    trace_to_std(&config).unwrap();
}

impl Distribution<ItemType> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ItemType {
        match rng.gen_range(0..5) {
            0 | 1 => ItemType::Category,
            2 => ItemType::Page,
            3 => ItemType::Product,
            4 => ItemType::Collection,
            _ => ItemType::Product,
        }
    }
}

pub fn testing_configs() -> (Config, SitemapConfig) {
    (
        Config {
            apl: AplType::File,
            apl_url: "redis://localhost:6379".to_string(),
            log_level: Level::TRACE,
            app_api_base_url: "http://localhost:3000".to_string(),
            app_iframe_base_url: "http://localhost:3000".to_string(),
            required_saleor_version: "^3.13".to_string(),
        },
        SitemapConfig {
            allowed_host: "https://api.example.com".to_string(),
            target_folder: "./temp/sitemaps".to_string(),
            pages_template: "https://example.com/{page.slug}".to_string(),
            index_hostname: "https://example.com".to_string(),
            product_template: "https://example.com/{product.category.slug}/{product.slug}"
                .to_string(),
            category_template: "https://example.com/{category.slug}".to_string(),
            collection_template: "https://example.com/collection/{collection.slug}".to_string(),
        },
    )
}

pub async fn create_query(
    mut app: RouterIntoService<Body>,
    body: String,
    webhook: EitherWebhookType,
) -> RouterIntoService<Body> {
    let response = app
        .ready()
        .await
        .unwrap()
        .call(
            Request::builder()
                .uri("/api/webhooks")
                .header(SALEOR_API_URL_HEADER, "https://api.example.com")
                .header(
                    SALEOR_EVENT_HEADER,
                    match webhook {
                        EitherWebhookType::Sync(s) => s.as_ref().to_string(),
                        EitherWebhookType::Async(a) => a.as_ref().to_string(),
                    },
                )
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    app
}

pub struct Action {
    request_body: String,
    url: Url,
    webhook_type: EitherWebhookType,
    action_type: ActionType,
}

#[derive(PartialEq, Eq, Clone)]
pub enum ActionType {
    Create,
    Update,
    Delete,
}

impl Distribution<ActionType> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ActionType {
        match rng.gen_range(0..4) {
            1 => ActionType::Update,
            2 => ActionType::Delete,
            _ => ActionType::Create,
        }
    }
}

// pub fn gen_random_actions(
//     len: usize,
//     sitemap_config: &SitemapConfig,
//     unwanted_actions: Vec<ActionType>,
// ) -> Vec<Action> {
//     let mut res: Vec<Action> = vec![];
//     for _ in 0..len {
//         let mut slug = random_word::gen(random_word::Lang::En).to_owned();
//         let mut id = cynic::Id::new(slug.to_owned() + "_ID");
//
//         let mut rel_slug = random_word::gen(random_word::Lang::En).to_owned();
//         let mut rel_id = cynic::Id::new(rel_slug.to_owned() + "_ID");
//
//         let mut action_type = rand::random::<ActionType>();
//
//         while unwanted_actions.contains(&action_type) {
//             action_type = rand::random::<ActionType>();
//         }
//
//         let item_type = rand::random::<ItemType>();
//         // If there is a category url already, use that for relation instead of always a
//         let mut is_using_existing_category = false;
//         // new one
//         if res
//             .iter()
//             .find(|r| r.url.data.typ == ItemType::Category)
//             .is_some()
//         {
//             match rand::random::<bool>() {
//                 true => loop {
//                     let r = res.choose(&mut rand::thread_rng()).unwrap().clone();
//                     if r.url.data.typ == ItemType::Category {
//                         rel_slug = r.url.data.slug;
//                         rel_id = cynic::Id::new(r.url.data.id);
//                         is_using_existing_category = true;
//                         break;
//                     }
//                 },
//                 false => (),
//             };
//         }
//         let body_data: String = match (action_type, item_type) {
//             (ActionType::Create, ItemType::Product) => {
//                 serde_json::to_string_pretty(&ProductCreated {
//                     product: Some(Product {
//                         id: id.clone(),
//                         slug: slug.clone(),
//                         category: Some(Category {
//                             slug: rel_slug.clone(),
//                             id: rel_id.clone(),
//                         }),
//                     }),
//                 })
//                 .unwrap()
//             }
//             (ActionType::Update, ItemType::Product) => {
//                 let p;
//                 loop {
//                     let c = res.choose(&mut rand::thread_rng()).unwrap().clone();
//                     if c.action_type != ActionType::Delete {
//                         p = c;
//                         break;
//                     }
//                 }
//                 serde_json::to_string_pretty(&ProductUpdated {
//                     product: Some(Product {
//                         id: cynic::Id::new(p.url.data.id),
//                         slug: p.url.data.slug.clone(),
//                         category: p.url.related.map(|c| Category {
//                             slug: c.slug.clone(),
//                             id: cynic::Id::new(c.id),
//                         }),
//                     }),
//                 })
//             }
//             (ActionType::Delete, ) => {}
//         };
//         let url = Url::new(
//             product_updated.clone(),
//             &sitemap_config,
//             ItemData {
//                 id: id.clone().inner().to_owned(),
//                 slug: slug.clone(),
//                 typ: ItemType::Product,
//             },
//             Some(ItemData {
//                 id: rel_id.inner().to_owned(),
//                 slug: rel_slug.clone(),
//                 typ: ItemType::Category,
//             }),
//         )
//         .unwrap();
//
//         if !is_using_existing_category {
//             let category_updated = CategoryUpdated {
//                 category: Some(Category2 {
//                     id: rel_id.clone(),
//                     slug: rel_slug.clone(),
//                 }),
//             };
//
//             let cat_url = Url::new(
//                 category_updated.clone(),
//                 &sitemap_config,
//                 ItemData {
//                     id: id.clone().inner().to_owned(),
//                     slug: slug.clone(),
//                     typ: ItemType::Category,
//                 },
//                 None,
//             )
//             .unwrap();
//             res.push((
//                 serde_json::to_string_pretty(&category_updated).unwrap(),
//                 cat_url,
//                 EitherWebhookType::Async(AsyncWebhookEventType::CategoryCreated),
//             ));
//         }
//
//         res.push((
//             serde_json::to_string_pretty(&product_updated).unwrap(),
//             url,
//             EitherWebhookType::Async(AsyncWebhookEventType::ProductCreated),
//         ));
//     }
//     res
// }
//
pub fn gen_random_url_set(
    len: usize,
    sitemap_config: &SitemapConfig,
) -> Vec<(String, Url, EitherWebhookType)> {
    let mut res: Vec<(String, Url, EitherWebhookType)> = vec![];
    for _ in 0..len {
        let slug = random_word::gen(random_word::Lang::En).to_owned();
        let id = cynic::Id::new(slug.to_owned() + "_ID");

        let mut rel_slug = random_word::gen(random_word::Lang::En).to_owned();
        let mut rel_id = cynic::Id::new(rel_slug.to_owned() + "_ID");

        match rand::random::<ItemType>() {
            ItemType::Product => {
                // If there is a category url already, use that for relation instead of always a
                let mut is_using_existing_category = false;
                // new one
                if res.iter().any(|r| r.1.data.typ == ItemType::Category) {
                    match rand::random::<bool>() {
                        true => loop {
                            let r = res.choose(&mut rand::thread_rng()).unwrap().clone();
                            if r.1.data.typ == ItemType::Category {
                                rel_slug = r.1.data.slug;
                                rel_id = cynic::Id::new(r.1.data.id);
                                is_using_existing_category = true;
                                break;
                            }
                        },
                        false => (),
                    };
                }
                let product_updated = ProductUpdated {
                    product: Some(Product {
                        id: id.clone(),
                        slug: slug.clone(),
                        category: Some(Category {
                            slug: rel_slug.clone(),
                            id: rel_id.clone(),
                        }),
                    }),
                };
                let url = Url::new(
                    product_updated.clone(),
                    sitemap_config,
                    ItemData {
                        id: id.clone().inner().to_owned(),
                        slug: slug.clone(),
                        typ: ItemType::Product,
                    },
                    Some(ItemData {
                        id: rel_id.inner().to_owned(),
                        slug: rel_slug.clone(),
                        typ: ItemType::Category,
                    }),
                )
                .unwrap();

                if !is_using_existing_category {
                    let category_updated = CategoryUpdated {
                        category: Some(Category2 {
                            id: rel_id.clone(),
                            slug: rel_slug.clone(),
                        }),
                    };

                    let cat_url = Url::new(
                        category_updated.clone(),
                        sitemap_config,
                        ItemData {
                            id: id.clone().inner().to_owned(),
                            slug: slug.clone(),
                            typ: ItemType::Category,
                        },
                        None,
                    )
                    .unwrap();
                    res.push((
                        serde_json::to_string_pretty(&category_updated).unwrap(),
                        cat_url,
                        EitherWebhookType::Async(AsyncWebhookEventType::CategoryCreated),
                    ));
                }

                res.push((
                    serde_json::to_string_pretty(&product_updated).unwrap(),
                    url,
                    EitherWebhookType::Async(AsyncWebhookEventType::ProductCreated),
                ));
            }
            ItemType::Category => {
                let category_updated = CategoryUpdated {
                    category: Some(Category2 {
                        id: id.clone(),
                        slug: slug.clone(),
                    }),
                };

                let url = Url::new(
                    category_updated.clone(),
                    sitemap_config,
                    ItemData {
                        id: id.clone().inner().to_owned(),
                        slug: slug.clone(),
                        typ: ItemType::Category,
                    },
                    None,
                )
                .unwrap();
                res.push((
                    serde_json::to_string_pretty(&category_updated).unwrap(),
                    url,
                    EitherWebhookType::Async(AsyncWebhookEventType::CategoryCreated),
                ));
            }
            ItemType::Collection => {
                let collection_updated = CollectionUpdated {
                    collection: Some(Collection {
                        id: id.clone(),
                        slug: slug.clone(),
                    }),
                };

                let url = Url::new(
                    collection_updated.clone(),
                    sitemap_config,
                    ItemData {
                        id: id.clone().inner().to_owned(),
                        slug: slug.clone(),
                        typ: ItemType::Collection,
                    },
                    None,
                )
                .unwrap();
                res.push((
                    serde_json::to_string_pretty(&collection_updated).unwrap(),
                    url,
                    EitherWebhookType::Async(AsyncWebhookEventType::CollectionCreated),
                ));
            }
            ItemType::Page => {
                let page_updated = PageUpdated {
                    page: Some(Page {
                        id: id.clone(),
                        slug: slug.clone(),
                    }),
                };

                let url = Url::new(
                    page_updated.clone(),
                    sitemap_config,
                    ItemData {
                        id: id.clone().inner().to_owned(),
                        slug: slug.clone(),
                        typ: ItemType::Page,
                    },
                    None,
                )
                .unwrap();
                res.push((
                    serde_json::to_string_pretty(&page_updated).unwrap(),
                    url,
                    EitherWebhookType::Async(AsyncWebhookEventType::PageCreated),
                ));
            }
        }
    }
    res
}
