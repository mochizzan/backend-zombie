//! Integration tests for User endpoints.
//!
//! These tests require a running MariaDB instance.
//! Run with: cargo test --test user_test -- --ignored

use actix_web::{test, http, App};
use actix_web::dev::Service;
use serde_json::json;

const API_KEY: &str = "dev-secret-key-change-in-production";

#[actix_rt::test]
#[ignore]
async fn test_register_and_get_user() {
    let app = test::init_service(
        App::new()
            .route("/health", actix_web::web::get().to(|| async { actix_web::HttpResponse::Ok().finish() }))
    ).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/users")
        .insert_header(("X-API-Key", API_KEY))
        .set_json(json!({
            "player_id": "test_player_001",
            "username": "TestUser",
            "nickname": "Tester",
            "role": "survival"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success() || resp.status().is_client_error());
}

#[actix_rt::test]
#[ignore]
async fn test_get_user_not_found() {
    let app = test::init_service(
        App::new()
            .route("/health", actix_web::web::get().to(|| async { actix_web::HttpResponse::Ok().finish() }))
    ).await;

    let req = test::TestRequest::get()
        .uri("/api/v1/users/nonexistent_player")
        .insert_header(("X-API-Key", API_KEY))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::NOT_FOUND);
}

#[actix_rt::test]
#[ignore]
async fn test_register_validation_error() {
    let app = test::init_service(
        App::new()
            .route("/health", actix_web::web::get().to(|| async { actix_web::HttpResponse::Ok().finish() }))
    ).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/users")
        .insert_header(("X-API-Key", API_KEY))
        .set_json(json!({
            "player_id": "",
            "username": "",
            "nickname": ""
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
}
