use apify_connector::actors::{get_actor_metadata, list_available_actors, ActorMetadata};
use axum::{extract::Path, http::StatusCode, routing::get, Json, Router};
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

fn app() -> Router {
    Router::new()
        .route("/actors", get(list_actors))
        .route("/actors/{actor_type}", get(get_actor_schema))
}

async fn list_actors() -> Json<Vec<ActorMetadata>> {
    Json(list_available_actors())
}

async fn get_actor_schema(Path(actor_type): Path<String>) -> Result<Json<ActorMetadata>, StatusCode> {
    get_actor_metadata(&actor_type)
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

#[tokio::test]
async fn test_list_actors_returns_all_actors() {
    let app = app();

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/actors")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let actors: Vec<Value> = serde_json::from_slice(&body).unwrap();

    // Verify we have all 4 actors
    assert_eq!(actors.len(), 4);

    // Verify actor types
    let actor_types: Vec<&str> = actors
        .iter()
        .map(|a| a["actor_type"].as_str().unwrap())
        .collect();
    assert!(actor_types.contains(&"web_scraper"));
    assert!(actor_types.contains(&"google_search"));
    assert!(actor_types.contains(&"instagram"));
    assert!(actor_types.contains(&"tripadvisor"));
}

#[tokio::test]
async fn test_web_scraper_schema_has_required_fields() {
    let app = app();

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/actors")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let actors: Vec<Value> = serde_json::from_slice(&body).unwrap();

    let web_scraper = actors
        .iter()
        .find(|a| a["actor_type"] == "web_scraper")
        .expect("web_scraper not found");

    assert_eq!(web_scraper["actor_name"], "apify/web-scraper");

    // Check schema has properties
    let props = &web_scraper["schema"]["properties"];
    assert!(props.get("startUrls").is_some(), "missing startUrls");
    assert!(props.get("maxPages").is_some(), "missing maxPages");
    assert!(
        props.get("contentSelector").is_some(),
        "missing contentSelector"
    );
    assert!(
        props.get("useApifyProxy").is_some(),
        "missing useApifyProxy"
    );

    // Verify schema title and description from doc comments
    assert_eq!(web_scraper["schema"]["title"], "WebScraperConfig");
    assert!(web_scraper["schema"]["description"]
        .as_str()
        .unwrap()
        .contains("Scrapes web pages"));
}

#[tokio::test]
async fn test_google_search_schema_has_required_fields() {
    let app = app();

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/actors")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let actors: Vec<Value> = serde_json::from_slice(&body).unwrap();

    let google_search = actors
        .iter()
        .find(|a| a["actor_type"] == "google_search")
        .expect("google_search not found");

    assert_eq!(google_search["actor_name"], "apify/google-search-scraper");

    let props = &google_search["schema"]["properties"];
    assert!(props.get("queries").is_some(), "missing queries");
    assert!(props.get("maxResults").is_some(), "missing maxResults");
    assert!(props.get("language").is_some(), "missing language");
    assert!(props.get("countryCode").is_some(), "missing countryCode");
}

#[tokio::test]
async fn test_instagram_schema_has_required_fields() {
    let app = app();

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/actors")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let actors: Vec<Value> = serde_json::from_slice(&body).unwrap();

    let instagram = actors
        .iter()
        .find(|a| a["actor_type"] == "instagram")
        .expect("instagram not found");

    assert_eq!(instagram["actor_name"], "apify/instagram-scraper");

    let props = &instagram["schema"]["properties"];
    assert!(props.get("usernames").is_some(), "missing usernames");
    assert!(props.get("maxPosts").is_some(), "missing maxPosts");
    assert!(
        props.get("includeProfileInfo").is_some(),
        "missing includeProfileInfo"
    );
    assert!(
        props.get("includeComments").is_some(),
        "missing includeComments"
    );
}

#[tokio::test]
async fn test_tripadvisor_schema_has_required_fields() {
    let app = app();

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/actors")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let actors: Vec<Value> = serde_json::from_slice(&body).unwrap();

    let tripadvisor = actors
        .iter()
        .find(|a| a["actor_type"] == "tripadvisor")
        .expect("tripadvisor not found");

    assert_eq!(tripadvisor["actor_name"], "Hvp4YfFGyLM635Q2F");

    let props = &tripadvisor["schema"]["properties"];
    assert!(props.get("url").is_some(), "missing url");
    assert!(props.get("reviewRatings").is_some(), "missing reviewRatings");
    assert!(
        props.get("reviewsLanguages").is_some(),
        "missing reviewsLanguages"
    );
    assert!(props.get("maxReviews").is_some(), "missing maxReviews");
    assert!(
        props.get("includeReviewerInfo").is_some(),
        "missing includeReviewerInfo"
    );
}

#[tokio::test]
async fn test_schema_includes_field_descriptions() {
    let app = app();

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/actors")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let actors: Vec<Value> = serde_json::from_slice(&body).unwrap();

    let web_scraper = actors
        .iter()
        .find(|a| a["actor_type"] == "web_scraper")
        .expect("web_scraper not found");

    // Check that field descriptions from doc comments are included
    let start_urls_desc = web_scraper["schema"]["properties"]["startUrls"]["description"]
        .as_str()
        .unwrap();
    assert!(
        start_urls_desc.contains("URLs"),
        "startUrls should have description"
    );
}

#[tokio::test]
async fn test_get_single_actor_schema() {
    let app = app();

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/actors/tripadvisor")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let actor: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(actor["actor_type"], "tripadvisor");
    assert_eq!(actor["actor_name"], "Hvp4YfFGyLM635Q2F");

    let props = &actor["schema"]["properties"];
    assert!(props.get("url").is_some(), "missing url");
    assert!(props.get("reviewRatings").is_some(), "missing reviewRatings");
}

#[tokio::test]
async fn test_get_single_actor_schema_not_found() {
    let app = app();

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/actors/unknown_actor")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_all_actor_types_individually() {
    let actor_types = ["web_scraper", "google_search", "instagram", "tripadvisor"];

    for actor_type in actor_types {
        let app = app();

        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri(format!("/actors/{}", actor_type))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Failed for actor_type: {}",
            actor_type
        );

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let actor: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(
            actor["actor_type"].as_str().unwrap(),
            actor_type,
            "actor_type mismatch"
        );
        assert!(
            actor["schema"]["properties"].is_object(),
            "schema should have properties"
        );
    }
}
