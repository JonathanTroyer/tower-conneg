#![cfg(all(feature = "hyper-client", feature = "json"))]

mod common;

use std::convert::Infallible;
use std::sync::Arc;

use bytes::Bytes;
use http::{Request, Response, header};
use http_body_util::Full;
use tower::{Layer, Service, ServiceExt};
use tower_conneg::{ClientConfig, ClientNegotiateLayer, ErasedFormat, HyperClientExt};

use common::JsonFormat;

type MockRequest = Request<Full<Bytes>>;
type MockResponse = Response<Full<Bytes>>;

fn mock_service_capturing_headers() -> (
    impl Service<MockRequest, Response = MockResponse, Error = Infallible, Future: Send> + Clone,
    Arc<std::sync::Mutex<Vec<(String, String)>>>,
) {
    let captured = Arc::new(std::sync::Mutex::new(Vec::new()));
    let captured_clone = captured.clone();

    let svc = tower::service_fn(move |req: MockRequest| {
        let captured = captured_clone.clone();
        async move {
            if let Ok(mut guard) = captured.lock() {
                if let Some(ct) = req.headers().get(header::CONTENT_TYPE) {
                    if let Ok(s) = ct.to_str() {
                        guard.push(("content-type".to_string(), s.to_string()));
                    }
                }
                if let Some(accept) = req.headers().get(header::ACCEPT) {
                    if let Ok(s) = accept.to_str() {
                        guard.push(("accept".to_string(), s.to_string()));
                    }
                }
            }
            Ok::<_, Infallible>(Response::new(Full::new(Bytes::new())))
        }
    });

    (svc, captured)
}

#[derive(Debug)]
struct TestError(String);

impl std::fmt::Display for TestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for TestError {}

impl From<Infallible> for TestError {
    fn from(e: Infallible) -> Self {
        match e {}
    }
}

impl<T> From<std::sync::PoisonError<T>> for TestError {
    fn from(e: std::sync::PoisonError<T>) -> Self {
        TestError(e.to_string())
    }
}

type TestResult = Result<(), TestError>;

// =============================================================================
// HyperClientExt Tests
// =============================================================================

#[tokio::test]
async fn with_content_negotiation_applies_layer() -> TestResult {
    let json: Arc<dyn ErasedFormat> = Arc::new(JsonFormat);

    let config = ClientConfig::builder()
        .formats([json.clone()])
        .fallback_format(json.clone())
        .build();

    let (inner, captured) = mock_service_capturing_headers();
    let mut client = inner.with_content_negotiation(config);

    let req = Request::builder()
        .uri("/")
        .body(Full::new(Bytes::new()))
        .map_err(|e| TestError(e.to_string()))?;

    let _ = client.ready().await?.call(req).await?;

    let headers = captured.lock()?;
    let content_types: Vec<_> = headers
        .iter()
        .filter(|(k, _)| k == "content-type")
        .collect();
    assert_eq!(content_types.len(), 1);
    assert_eq!(content_types[0].1, "application/json");

    Ok(())
}

#[tokio::test]
async fn with_content_negotiation_sets_accept_header() -> TestResult {
    let json: Arc<dyn ErasedFormat> = Arc::new(JsonFormat);

    let config = ClientConfig::builder()
        .formats([json.clone()])
        .fallback_format(json.clone())
        .build();

    let (inner, captured) = mock_service_capturing_headers();
    let mut client = inner.with_content_negotiation(config);

    let req = Request::builder()
        .uri("/")
        .body(Full::new(Bytes::new()))
        .map_err(|e| TestError(e.to_string()))?;

    let _ = client.ready().await?.call(req).await?;

    let headers = captured.lock()?;
    let accept_headers: Vec<_> = headers.iter().filter(|(k, _)| k == "accept").collect();
    assert_eq!(accept_headers.len(), 1);
    assert!(accept_headers[0].1.contains("application/json"));

    Ok(())
}

#[tokio::test]
async fn extension_trait_equivalent_to_layer() -> TestResult {
    let json: Arc<dyn ErasedFormat> = Arc::new(JsonFormat);

    let config = ClientConfig::builder()
        .formats([json.clone()])
        .fallback_format(json.clone())
        .build();

    // Using extension trait
    let (inner1, captured1) = mock_service_capturing_headers();
    let mut client1 = inner1.with_content_negotiation(config.clone());

    // Using layer directly
    let (inner2, captured2) = mock_service_capturing_headers();
    let mut client2 = ClientNegotiateLayer::new(config).layer(inner2);

    let req1 = Request::builder()
        .uri("/")
        .body(Full::new(Bytes::new()))
        .map_err(|e| TestError(e.to_string()))?;

    let req2 = Request::builder()
        .uri("/")
        .body(Full::new(Bytes::new()))
        .map_err(|e| TestError(e.to_string()))?;

    let _ = client1.ready().await?.call(req1).await?;
    let _ = client2.ready().await?.call(req2).await?;

    let headers1 = captured1.lock()?;
    let headers2 = captured2.lock()?;

    assert_eq!(headers1.len(), headers2.len());
    for (h1, h2) in headers1.iter().zip(headers2.iter()) {
        assert_eq!(h1, h2);
    }

    Ok(())
}

#[tokio::test]
async fn format_caching_works_through_extension() -> TestResult {
    let json: Arc<dyn ErasedFormat> = Arc::new(JsonFormat);

    let config = ClientConfig::builder()
        .formats([json.clone()])
        .fallback_format(json.clone())
        .build();

    let (inner, _captured) = mock_service_capturing_headers();
    let mut client = inner.with_content_negotiation(config);

    assert!(client.cached_format().is_none());

    let req = Request::builder()
        .uri("/")
        .body(Full::new(Bytes::new()))
        .map_err(|e| TestError(e.to_string()))?;

    let _ = client.ready().await?.call(req).await?;

    let cached = client.cached_format();
    assert!(cached.is_some());
    let format = cached.ok_or_else(|| TestError("no cached format".to_string()))?;
    assert_eq!(format.content_type_header(), "application/json");

    Ok(())
}
