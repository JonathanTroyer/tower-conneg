#![cfg(all(feature = "poem", feature = "json"))]
#![allow(missing_docs)]

use std::sync::Arc;

use poem::http::{StatusCode, header};
use poem::test::TestClient;
use poem::{Endpoint, EndpointExt, Request, Result, Route, handler};
use serde::{Deserialize, Serialize};
use tower_conneg::{ErasedFormat, JsonFormat, Negotiate, NegotiateResponse, NegotiatedFormat};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestData {
    value: String,
}

async fn negotiate_middleware<E: Endpoint>(next: E, mut req: Request) -> Result<E::Output> {
    let json: Arc<dyn ErasedFormat> = Arc::new(JsonFormat);

    let content_type = req
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok());

    let negotiated = match content_type {
        Some(ct) if ct.contains("application/json") => {
            NegotiatedFormat::with_request_format(json.clone(), json)
        }
        _ => NegotiatedFormat::response_only(json),
    };

    req.extensions_mut().insert(negotiated);
    next.call(req).await
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

#[tokio::test]
async fn negotiate_unit_extraction() {
    let app = Route::new()
        .at("/", poem::get(get_handler))
        .around(negotiate_middleware);

    let cli = TestClient::new(app);
    let resp = cli
        .get("/")
        .header(header::ACCEPT, "application/json")
        .send()
        .await;

    resp.assert_status_is_ok();
    resp.assert_header(header::CONTENT_TYPE, "application/json");
}

#[tokio::test]
async fn negotiate_body_extraction() {
    let app = Route::new()
        .at("/", poem::post(post_handler))
        .around(negotiate_middleware);

    let cli = TestClient::new(app);
    let resp = cli
        .post("/")
        .header(header::ACCEPT, "application/json")
        .body_json(&TestData {
            value: "test".to_string(),
        })
        .send()
        .await;

    resp.assert_status_is_ok();
    let body: TestData = resp.json().await.value().deserialize();
    assert_eq!(body.value, "received: test");
}

#[tokio::test]
async fn negotiate_response_serialization() {
    let app = Route::new()
        .at("/", poem::get(get_handler))
        .around(negotiate_middleware);

    let cli = TestClient::new(app);
    let resp = cli
        .get("/")
        .header(header::ACCEPT, "application/json")
        .send()
        .await;

    resp.assert_status_is_ok();
    resp.assert_header(header::CONTENT_TYPE, "application/json");
    let body: TestData = resp.json().await.value().deserialize();
    assert_eq!(body.value, "hello");
}

#[tokio::test]
async fn roundtrip_json() {
    let app = Route::new()
        .at("/", poem::post(post_handler))
        .around(negotiate_middleware);

    let cli = TestClient::new(app);

    let input = TestData {
        value: "roundtrip test".to_string(),
    };

    let resp = cli
        .post("/")
        .header(header::ACCEPT, "application/json")
        .body_json(&input)
        .send()
        .await;

    resp.assert_status_is_ok();
    resp.assert_header(header::CONTENT_TYPE, "application/json");
    let output: TestData = resp.json().await.value().deserialize();
    assert_eq!(output.value, "received: roundtrip test");
}

#[tokio::test]
async fn missing_middleware_error() {
    #[handler]
    async fn handler(_neg: Negotiate<()>) -> NegotiateResponse<TestData> {
        unreachable!("Handler should not be called")
    }

    let app = Route::new().at("/", poem::get(handler));

    let cli = TestClient::new(app);
    let resp = cli
        .get("/")
        .header(header::ACCEPT, "application/json")
        .send()
        .await;

    resp.assert_status(StatusCode::NOT_ACCEPTABLE);
}
