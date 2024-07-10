use crate::{
    app::{trace_to_std, SitemapConfig},
    create_app,
    queries::event_subjects_updated::{Category, Product, ProductUpdated},
    sitemap::{RefType, Url, UrlSet},
};
use axum::{
    body::Body,
    extract::path::ErrorKind,
    http::{Request, StatusCode},
    routing::RouterIntoService,
    Json, Router,
};
use rstest::*;
use saleor_app_sdk::{apl::AplType, config::Config};
use tower::{MakeService, Service, ServiceExt};
use tracing::Level;

fn init_tracing() {
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

async fn init_test_app() -> RouterIntoService<Body> {
    match std::fs::remove_dir_all("./temp/sitemaps") {
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => (),
            _ => panic!("{:?}", e),
        },
        _ => (),
    };
    std::fs::create_dir_all("./temp/sitemaps").unwrap();

    std::env::set_var("APP_API_BASE_URL", "http://localhost:3000");

    let config = Config {
        apl: AplType::File,
        apl_url: "redis://localhost:6379".to_string(),
        log_level: Level::TRACE,
        app_api_base_url: "http://localhost:3000".to_string(),
        app_iframe_base_url: "http://localhost:3000".to_string(),
        required_saleor_version: "^3.13".to_string(),
    };
    let sitemap_config = SitemapConfig {
        target_folder: "./temp/sitemaps".to_string(),
        pages_template: "https://example.com/{page.slug}".to_string(),
        index_hostname: "https://example.com".to_string(),
        product_template: "https://example.com/{product.category.slug}/{product.slug}".to_string(),
        category_template: "https://example.com/{category.slug}".to_string(),
        collection_template: "https://example.com/collection/{collection.slug}".to_string(),
    };

    create_app(&config, sitemap_config)
        .await
        .into_service::<Body>()
}

#[rstest]
async fn index_returns_ok() {
    let mut app = init_test_app().await;

    let response = app
        .ready()
        .await
        .unwrap()
        .call(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[rstest]
async fn updates_xml_from_product() {
    let mut app = init_test_app().await;
    // let app = app.ready().await.unwrap();

    let product_id = cynic::Id::new("product1".to_owned());
    let product_slug = "product1slug".to_owned();
    let category_id = cynic::Id::new("category1".to_owned());
    let category_slug = "category1slug".to_owned();

    let response = app
        .ready()
        .await
        .unwrap()
        .call(
            Request::builder()
                .uri("/api/webhooks")
                .body(Body::from(
                    serde_json::to_string_pretty(&ProductUpdated {
                        product: Some(Product {
                            id: product_id.clone(),
                            slug: product_slug.clone(),
                            category: Some(Category {
                                slug: category_slug.clone(),
                                id: category_id.clone(),
                            }),
                        }),
                    })
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let xml: UrlSet =
        serde_json::from_str(&std::fs::read_to_string("./temp/sitemaps/1.xml").unwrap()).unwrap();

    let mut webhook_url_set = UrlSet::new();
    webhook_url_set.url = vec![Url::new_with_ref(
        product_id.inner().to_owned(),
        product_slug.clone(),
        RefType::Product,
        Some(category_id.inner().to_owned()),
        Some(category_slug.clone()),
        Some(RefType::Category),
    )];

    assert_eq!(xml, webhook_url_set);
}

#[rstest]
fn urlset_serialisation_isnt_lossy() {
    std::env::set_var("APP_API_BASE_URL", "http://localhost:3000");
    init_tracing();
    let mut url_set = UrlSet::new();
    url_set.url.append(&mut vec![
        Url::new(
            "category1coolid".to_string(),
            "category1".to_string(),
            RefType::Category,
        ),
        Url::new(
            "Collection1coolid".to_string(),
            "Collection1".to_string(),
            RefType::Collection,
        ),
        Url::new_with_ref(
            "category1coolid".to_string(),
            "category1".to_string(),
            RefType::Product,
            Some("product1coolid".to_string()),
            Some("product1".to_string()),
            Some(RefType::Category),
        ),
        Url::new_with_ref(
            "category2coolid".to_string(),
            "category2".to_string(),
            RefType::Product,
            Some("product2coolid".to_string()),
            Some("product2".to_string()),
            Some(RefType::Category),
        ),
    ]);
    let file_str = quick_xml::se::to_string(&url_set).unwrap();
    dbg!(&file_str);
    let deserialized_url_set: UrlSet = quick_xml::de::from_str(&file_str).unwrap();
    assert_eq!(url_set, deserialized_url_set);
}
