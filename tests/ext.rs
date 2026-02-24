//! Tests for the extension traits.

#![cfg(all(feature = "json", feature = "xml"))]

use std::sync::Arc;

use bytes::Bytes;
use http::{Request, Response, StatusCode, header};
use http_body_util::Full;
use serde::{Deserialize, Serialize};
use tower_conneg::{
    ErasedFormat, NegotiateRequestBuilderExt, NegotiateResponseExt, NegotiationError,
};

mod common;
use common::{JsonFormat, XmlFormat};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct TestData {
    name: String,
    value: u32,
}

#[test]
fn accept_formats_sets_header_with_quality_values() {
    let json: Arc<dyn ErasedFormat> = Arc::new(JsonFormat);
    let xml: Arc<dyn ErasedFormat> = Arc::new(XmlFormat);
    let formats = vec![json, xml];

    let request = Request::builder()
        .method("GET")
        .uri("/test")
        .accept_formats(&formats)
        .body(Full::new(Bytes::new()))
        .unwrap();

    let accept = request.headers().get(header::ACCEPT).unwrap();
    let accept_str = accept.to_str().unwrap();

    assert!(accept_str.contains("application/json"));
    assert!(accept_str.contains("application/xml"));
    assert!(accept_str.contains(";q=0.9"));
}

#[test]
fn accept_formats_handles_empty_list() {
    let formats: Vec<Arc<dyn ErasedFormat>> = vec![];

    let request = Request::builder()
        .method("GET")
        .uri("/test")
        .accept_formats(&formats)
        .body(Full::new(Bytes::new()))
        .unwrap();

    assert!(request.headers().get(header::ACCEPT).is_none());
}

#[test]
fn accept_formats_single_format_no_quality() {
    let json: Arc<dyn ErasedFormat> = Arc::new(JsonFormat);
    let formats = vec![json];

    let request = Request::builder()
        .method("GET")
        .uri("/test")
        .accept_formats(&formats)
        .body(Full::new(Bytes::new()))
        .unwrap();

    let accept = request.headers().get(header::ACCEPT).unwrap();
    let accept_str = accept.to_str().unwrap();

    assert_eq!(accept_str, "application/json");
    assert!(!accept_str.contains(";q="));
}

#[tokio::test]
async fn body_with_format_serializes_and_sets_content_type() {
    let json = JsonFormat;
    let data = TestData {
        name: "test".to_string(),
        value: 42,
    };

    let request = Request::builder()
        .method("POST")
        .uri("/test")
        .body_with_format(&data, &json)
        .unwrap();

    let content_type = request.headers().get(header::CONTENT_TYPE).unwrap();
    assert_eq!(content_type, "application/json");

    let (_, body) = request.into_parts();
    use http_body_util::BodyExt;
    let collected = body.collect().await.unwrap().to_bytes();
    let parsed: TestData = serde_json::from_slice(&collected).unwrap();
    assert_eq!(parsed, data);
}

#[tokio::test]
async fn negotiated_format_matches_content_type() {
    let json: Arc<dyn ErasedFormat> = Arc::new(JsonFormat);
    let xml: Arc<dyn ErasedFormat> = Arc::new(XmlFormat);
    let formats = vec![json, xml];

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Full::new(Bytes::new()))
        .unwrap();

    let format = response.negotiated_format(&formats).unwrap();
    assert_eq!(
        format.content_type_header().to_str().unwrap(),
        "application/json"
    );
}

#[tokio::test]
async fn negotiated_format_returns_error_for_unknown_content_type() {
    let json: Arc<dyn ErasedFormat> = Arc::new(JsonFormat);
    let formats = vec![json];

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/plain")
        .body(Full::new(Bytes::new()))
        .unwrap();

    let result = response.negotiated_format(&formats);
    assert!(matches!(
        result,
        Err(NegotiationError::UnsupportedMediaType { .. })
    ));
}

#[tokio::test]
async fn negotiated_format_returns_error_for_missing_content_type() {
    let json: Arc<dyn ErasedFormat> = Arc::new(JsonFormat);
    let formats = vec![json];

    let response = Response::builder()
        .status(StatusCode::OK)
        .body(Full::new(Bytes::new()))
        .unwrap();

    let result = response.negotiated_format(&formats);
    assert!(matches!(
        result,
        Err(NegotiationError::UnsupportedMediaType { .. })
    ));
}

#[tokio::test]
async fn deserialize_collects_body_and_deserializes() {
    let json: Arc<dyn ErasedFormat> = Arc::new(JsonFormat);
    let formats = vec![json];

    let data = TestData {
        name: "response".to_string(),
        value: 123,
    };
    let body_bytes = serde_json::to_vec(&data).unwrap();

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Full::new(Bytes::from(body_bytes)))
        .unwrap();

    let result: TestData = response.deserialize(&formats).await.unwrap();
    assert_eq!(result, data);
}

#[tokio::test]
async fn deserialize_returns_error_for_invalid_json() {
    let json: Arc<dyn ErasedFormat> = Arc::new(JsonFormat);
    let formats = vec![json];

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Full::new(Bytes::from("not valid json")))
        .unwrap();

    let result: Result<TestData, _> = response.deserialize(&formats).await;
    assert!(matches!(
        result,
        Err(NegotiationError::Deserialization { .. })
    ));
}
