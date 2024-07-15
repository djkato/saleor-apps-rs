mod utils;

use std::time::Duration;

use crate::{create_app, sitemap::UrlSet};
use async_std::task::sleep;
use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::RouterIntoService,
};
use rstest::*;
use saleor_app_sdk::{
    headers::{SALEOR_API_URL_HEADER, SALEOR_EVENT_HEADER},
    webhooks::utils::EitherWebhookType,
};
use serial_test::{parallel, serial};
use tower::{Service, ServiceExt};
use tracing_test::traced_test;
use utils::{gen_random_url_set, testing_configs};

async fn init_test_app() -> RouterIntoService<Body> {
    if let Err(e) = std::fs::remove_dir_all("./temp/sitemaps") {
        match e.kind() {
            std::io::ErrorKind::NotFound => (),
            _ => panic!("{:?}", e),
        }
    };
    std::fs::create_dir_all("./temp/sitemaps").unwrap();
    std::env::set_var("APP_API_BASE_URL", "http://localhost:3000");
    let (config, sitemap_config) = testing_configs();

    create_app(&config, sitemap_config)
        .await
        .into_service::<Body>()
}

#[rstest]
#[tokio::test]
#[traced_test]
#[serial]
pub async fn index_returns_ok() {
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
#[tokio::test]
#[traced_test]
#[serial]
async fn updates_sitemap_from_request() {
    let mut app = init_test_app().await;
    let (_, sitemap_config) = testing_configs();

    let evn = gen_random_url_set(1, &sitemap_config);
    let (body, url, webhook_type) = evn.first().cloned().unwrap();

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
                    match webhook_type {
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

    //wait for the file to get written
    sleep(Duration::from_secs(3)).await;

    let file_url = std::fs::read_to_string("./temp/sitemaps/sitemap.txt").unwrap();

    assert_eq!(file_url, url.url);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[parallel]
async fn sequence_of_actions_is_preserved() {
    let mut app = init_test_app().await;
    let (_, sitemap_config) = testing_configs();

    let evn = gen_random_url_set(1000, &sitemap_config);
    for (body, _, webhook_type) in evn.clone() {
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
                        match webhook_type {
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
    }

    //wait for the file to get written
    sleep(Duration::from_secs(3)).await;

    let file_url = std::fs::read_to_string("./temp/sitemaps/sitemap.txt").unwrap();

    assert_eq!(
        file_url,
        evn.iter()
            .map(|u| u.1.url.clone())
            .collect::<Vec<_>>()
            .join("\n"),
    );
}

#[rstest]
#[traced_test]
#[parallel]
fn urlset_serialisation_isnt_lossy() {
    std::env::set_var("APP_API_BASE_URL", "http://localhost:3000");
    let (_, sitemap_config) = testing_configs();

    let urls = gen_random_url_set(100, &sitemap_config);

    let mut url_set = UrlSet::new();
    url_set.urls = urls.into_iter().map(|u| u.1).collect();
    let file_str = serde_cbor::to_vec(&url_set).unwrap();
    let deserialized_url_set: UrlSet = serde_cbor::de::from_slice(&file_str).unwrap();
    assert_eq!(url_set, deserialized_url_set);
}
//TODO: TEST UPDATES AND DELETES, UPDATING URL CREATES A NEW ENTRY INSTEAD OF EDITING PREVIOUS ONE

// #[rstest]
// #[traced_test]
// #[parallel]
// async fn url_set_find_affected_works() {
//     let mut url_set = get_db_from_file("./").await.unwrap();
//     assert!(url_set
//         .find_affected("UHJvZHVjdDoxNjEwMg==", "dute-vlakno-0-5kg-biele")
//         .is_empty());
//     assert!(!url_set
//         .find_affected("UHJvZHVjdDoxNjEwMg==", "dute-vlakno-0-5kg-biele-test")
//         .is_empty());
// }
