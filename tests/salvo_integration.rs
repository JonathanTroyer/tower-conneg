#![cfg(all(feature = "salvo", feature = "json"))]
#![allow(missing_docs)]

use std::sync::Arc;

use salvo::http::StatusCode;
use salvo::prelude::*;
use salvo::test::{ResponseExt, TestClient};
use serde::{Deserialize, Serialize};
use tower_conneg::{
    ErasedFormat, JsonFormat, Negotiate, NegotiateLayer, NegotiateResponse, ServerConfig,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestData {
    value: String,
}

fn build_config() -> ServerConfig {
    let json: Arc<dyn ErasedFormat> = Arc::new(JsonFormat);
    ServerConfig::builder()
        .formats(vec![json.clone()])
        .fallback_format(json)
        .build()
}

#[handler]
async fn get_handler(neg: Negotiate<()>) -> NegotiateResponse<TestData> {
    neg.respond(TestData {
        value: "hello".to_string(),
    })
}

#[handler]
async fn post_handler(req: Negotiate<TestData>) -> NegotiateResponse<TestData> {
    let format = Arc::clone(req.format());
    let mut data = req.into_inner();
    data.value = format!("received: {}", data.value);
    NegotiateResponse::new(data, format)
}

fn build_service_with_middleware() -> Service {
    let router = Router::new().push(Router::with_path("/").get(get_handler).post(post_handler));
    Service::new(router).hoop(NegotiateLayer::new(build_config()).compat())
}

fn build_service_without_middleware() -> Service {
    let router = Router::new().push(Router::with_path("/").get(get_handler));
    Service::new(router)
}

#[tokio::test]
async fn negotiate_unit_extraction() {
    let service = build_service_with_middleware();

    let response = TestClient::get("http://localhost/")
        .add_header("Accept", "application/json", true)
        .send(&service)
        .await;

    assert_eq!(response.status_code, Some(StatusCode::OK));

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v: &salvo::http::HeaderValue| v.to_str().ok());
    assert_eq!(content_type, Some("application/json"));
}

#[tokio::test]
async fn negotiate_body_extraction() {
    let service = build_service_with_middleware();

    let input = TestData {
        value: "test".to_string(),
    };

    let mut response = TestClient::post("http://localhost/")
        .add_header("Accept", "application/json", true)
        .add_header("Content-Type", "application/json", true)
        .json(&input)
        .send(&service)
        .await;

    assert_eq!(response.status_code, Some(StatusCode::OK));

    let body = response.take_string().await.unwrap_or_default();
    let data: TestData = serde_json::from_str(&body).expect("Failed to deserialize response");
    assert_eq!(data.value, "received: test");
}

#[tokio::test]
async fn negotiate_response_serialization() {
    let service = build_service_with_middleware();

    let mut response = TestClient::get("http://localhost/")
        .add_header("Accept", "application/json", true)
        .send(&service)
        .await;

    assert_eq!(response.status_code, Some(StatusCode::OK));

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v: &salvo::http::HeaderValue| v.to_str().ok());
    assert_eq!(content_type, Some("application/json"));

    let body = response.take_string().await.unwrap_or_default();
    let data: TestData = serde_json::from_str(&body).expect("Failed to deserialize response");
    assert_eq!(data.value, "hello");
}

#[tokio::test]
async fn roundtrip_json() {
    let service = build_service_with_middleware();

    let input = TestData {
        value: "roundtrip test".to_string(),
    };

    let mut response = TestClient::post("http://localhost/")
        .add_header("Accept", "application/json", true)
        .add_header("Content-Type", "application/json", true)
        .json(&input)
        .send(&service)
        .await;

    assert_eq!(response.status_code, Some(StatusCode::OK));

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v: &salvo::http::HeaderValue| v.to_str().ok());
    assert_eq!(content_type, Some("application/json"));

    let body = response.take_string().await.unwrap_or_default();
    let output: TestData = serde_json::from_str(&body).expect("Failed to deserialize response");
    assert_eq!(output.value, "received: roundtrip test");
}

#[tokio::test]
async fn missing_middleware_error() {
    let service = build_service_without_middleware();

    let response = TestClient::get("http://localhost/")
        .add_header("Accept", "application/json", true)
        .send(&service)
        .await;

    assert_eq!(response.status_code, Some(StatusCode::NOT_ACCEPTABLE));
}
