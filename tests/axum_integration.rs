#![cfg(all(feature = "axum", feature = "json"))]

use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::routing::{get, post};
use http::{Request, StatusCode, header};
use serde::{Deserialize, Serialize};
use tower::ServiceExt;
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

async fn get_handler(neg: Negotiate<()>) -> NegotiateResponse<TestData> {
    neg.respond(TestData {
        value: "hello".to_string(),
    })
}

async fn post_handler(req: Negotiate<TestData>) -> NegotiateResponse<TestData> {
    let format = Arc::clone(req.format());
    let mut data = req.into_inner();
    data.value = format!("received: {}", data.value);
    NegotiateResponse::new(data, format)
}

#[tokio::test]
async fn negotiate_unit_extraction() {
    let app = Router::new()
        .route("/", get(get_handler))
        .layer(NegotiateLayer::new(build_config()));

    let req = Request::builder()
        .method("GET")
        .uri("/")
        .header(header::ACCEPT, "application/json")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let content_type = response.headers().get(header::CONTENT_TYPE).unwrap();
    assert_eq!(content_type, "application/json");
}

#[tokio::test]
async fn negotiate_body_extraction() {
    let app = Router::new()
        .route("/", post(post_handler))
        .layer(NegotiateLayer::new(build_config()));

    let body = serde_json::to_string(&TestData {
        value: "test".to_string(),
    })
    .unwrap();

    let req = Request::builder()
        .method("POST")
        .uri("/")
        .header(header::ACCEPT, "application/json")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let data: TestData = serde_json::from_slice(&body).unwrap();
    assert_eq!(data.value, "received: test");
}

#[tokio::test]
async fn negotiate_response_serialization() {
    let app = Router::new()
        .route("/", get(get_handler))
        .layer(NegotiateLayer::new(build_config()));

    let req = Request::builder()
        .method("GET")
        .uri("/")
        .header(header::ACCEPT, "application/json")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let content_type = response.headers().get(header::CONTENT_TYPE).unwrap();
    assert_eq!(content_type, "application/json");

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let data: TestData = serde_json::from_slice(&body).unwrap();
    assert_eq!(data.value, "hello");
}

#[tokio::test]
async fn missing_middleware_error() {
    async fn handler(_neg: Negotiate<()>) -> NegotiateResponse<TestData> {
        unreachable!("Handler should not be called")
    }

    let app = Router::new().route("/", get(handler));

    let req = Request::builder()
        .method("GET")
        .uri("/")
        .header(header::ACCEPT, "application/json")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_ACCEPTABLE);
}

#[tokio::test]
async fn roundtrip_json() {
    let app = Router::new()
        .route("/", post(post_handler))
        .layer(NegotiateLayer::new(build_config()));

    let input = TestData {
        value: "roundtrip test".to_string(),
    };
    let body = serde_json::to_string(&input).unwrap();

    let req = Request::builder()
        .method("POST")
        .uri("/")
        .header(header::ACCEPT, "application/json")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let content_type = response.headers().get(header::CONTENT_TYPE).unwrap();
    assert_eq!(content_type, "application/json");

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let output: TestData = serde_json::from_slice(&body).unwrap();
    assert_eq!(output.value, "received: roundtrip test");
}

#[tokio::test]
async fn missing_content_type_for_body_request_returns_error() {
    let app = Router::new()
        .route("/", post(post_handler))
        .layer(NegotiateLayer::new(build_config()));

    let req = Request::builder()
        .method("POST")
        .uri("/")
        .header(header::ACCEPT, "application/json")
        .body(Body::from("{}"))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn error_response_format() {
    async fn handler(_neg: Negotiate<()>) -> NegotiateResponse<TestData> {
        unreachable!()
    }

    let app = Router::new().route("/", get(handler));

    let req = Request::builder()
        .method("GET")
        .uri("/")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_ACCEPTABLE);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(body_str.contains("no acceptable response format"));
}
