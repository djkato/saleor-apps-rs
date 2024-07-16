mod utils;

use std::time::Duration;

use crate::{
    create_app,
    queries::event_subjects_updated::{Category, Product, ProductUpdated},
    sitemap::{ItemType, Url, UrlSet},
};
use async_std::task::sleep;
use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::RouterIntoService,
};
use rstest::*;
use saleor_app_sdk::{
    headers::{SALEOR_API_URL_HEADER, SALEOR_EVENT_HEADER},
    webhooks::{utils::EitherWebhookType, AsyncWebhookEventType},
};
use serial_test::{parallel, serial};
use tower::{Service, ServiceExt};
use tracing::debug;
use tracing_test::traced_test;
use utils::{create_query, gen_random_url_set, testing_configs};

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
pub async fn app_runs_and_responses() {
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
async fn update_event_updates_correctly() {
    let mut app = init_test_app().await;
    let (_, sitemap_config) = testing_configs();

    let mut evn = gen_random_url_set(50, &sitemap_config);
    for (body, _, webhook_type) in evn.clone() {
        app = create_query(app, body, webhook_type).await;
    }

    //wait for the file to get written
    sleep(Duration::from_secs(1)).await;

    let file_url = std::fs::read_to_string("./temp/sitemaps/sitemap.txt").unwrap();

    assert_eq!(
        file_url,
        evn.iter()
            .map(|u| u.1.url.clone())
            .collect::<Vec<_>>()
            .join("\n"),
    );
    /* ======== Now update it and see if it changed correctly ========*/

    {
        let (_, update_1, _) = evn
            .iter_mut()
            .find(|e| e.1.data.typ == ItemType::Product)
            // 0.01785820902% chance this will crash, wanna bet? :D
            .expect("you rolled a 0.01785820902% chance just now, feel proud of yourself");

        //no nice way to do this, I control the templates in test anyways so whatever
        let q_1 = update_1.clone().into_event_updated_body("_UPDATED");
        debug!("{:?}", &q_1);
        update_1.data.slug = update_1.clone().data.slug + "_UPDATED";
        update_1.url = format!(
            "https://example.com/{}/{}",
            &update_1.related.as_ref().unwrap().slug,
            &update_1.data.slug
        );
        debug!("{:?}", &update_1.url);

        app = create_query(
            app,
            q_1.0,
            EitherWebhookType::Async(AsyncWebhookEventType::ProductUpdated),
        )
        .await;

        sleep(Duration::from_secs(1)).await;
        let file_url = std::fs::read_to_string("./temp/sitemaps/sitemap.txt").unwrap();
        assert_eq!(
            file_url,
            evn.clone()
                .iter()
                .map(|u| u.1.url.clone())
                .collect::<Vec<_>>()
                .join("\n"),
        );
    }

    /* ======== Now update a category and see if all products are correct ========*/

    let affected_id: String;
    let affected_slug: String;
    {
        let (_, update_2, _) = evn
            .iter_mut()
            .find(|e| e.1.data.typ == ItemType::Category)
            // 0.01785820902% chance this will crash, wanna bet? :D
            .expect("you rolled a 0.01785820902% chance just now, feel proud of yourself");

        //no nice way to do this, I control the templates in test anyways so whatever
        let q_2 = update_2.clone().into_event_updated_body("_UPDATED");
        debug!("{:?}", &q_2);
        app = create_query(
            app,
            q_2.0,
            EitherWebhookType::Async(AsyncWebhookEventType::CategoryUpdated),
        )
        .await;

        update_2.data.slug = update_2.clone().data.slug + "_UPDATED";
        update_2.url = format!("https://example.com/{}", &update_2.data.slug);
        debug!("{:?}", &update_2.url);
        affected_id = update_2.data.id.clone();
        affected_slug = update_2.data.slug.clone();
    }
    evn.iter_mut().for_each(|u| {
        if u.1.data.typ == ItemType::Product
            && u.1.related.as_ref().map_or(false, |c| c.id == affected_id)
        {
            u.1.url = format!("https://example.com/{}/{}", affected_slug, &u.1.data.slug);
        }
    });

    sleep(Duration::from_secs(1)).await;
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
    sleep(Duration::from_secs(1)).await;

    let file_url = std::fs::read_to_string("./temp/sitemaps/sitemap.txt").unwrap();

    assert_eq!(file_url, url.url);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[serial]
async fn sequence_of_actions_is_preserved() {
    let mut app = init_test_app().await;
    let (_, sitemap_config) = testing_configs();

    let evn = gen_random_url_set(10, &sitemap_config);
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
    sleep(Duration::from_secs(1)).await;

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
