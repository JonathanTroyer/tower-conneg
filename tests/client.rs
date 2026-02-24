#![cfg(all(feature = "json", feature = "xml"))]

mod common;

use std::convert::Infallible;
use std::sync::Arc;

use bytes::Bytes;
use http::{Request, Response, StatusCode, header};
use http_body_util::Full;
use tower::{Layer, Service, ServiceExt};
use tower_conneg::{ClientConfig, ClientNegotiateLayer, ClientRequestExt, ErasedFormat};

use common::{JsonFormat, XmlFormat};

const ACCEPT_POST: http::HeaderName = http::HeaderName::from_static("accept-post");
const ACCEPT_PATCH: http::HeaderName = http::HeaderName::from_static("accept-patch");

type MockRequest = Request<Full<Bytes>>;
type MockResponse = Response<Full<Bytes>>;

fn mock_ok_service()
-> impl Service<MockRequest, Response = MockResponse, Error = Infallible, Future: Send> + Clone {
    tower::service_fn(|_req: MockRequest| async { Ok(Response::new(Full::new(Bytes::new()))) })
}

fn mock_415_with_accept_post(
    accept_post: &'static str,
) -> impl Service<MockRequest, Response = MockResponse, Error = Infallible, Future: Send> + Clone {
    tower::service_fn(move |_req: MockRequest| async move {
        let response = Response::builder()
            .status(StatusCode::UNSUPPORTED_MEDIA_TYPE)
            .header(ACCEPT_POST, accept_post)
            .body(Full::new(Bytes::new()));
        match response {
            Ok(r) => Ok(r),
            Err(_) => Ok(Response::new(Full::new(Bytes::new()))),
        }
    })
}

fn mock_415_with_accept_patch(
    accept_patch: &'static str,
) -> impl Service<MockRequest, Response = MockResponse, Error = Infallible, Future: Send> + Clone {
    tower::service_fn(move |_req: MockRequest| async move {
        let response = Response::builder()
            .status(StatusCode::UNSUPPORTED_MEDIA_TYPE)
            .header(ACCEPT_PATCH, accept_patch)
            .body(Full::new(Bytes::new()));
        match response {
            Ok(r) => Ok(r),
            Err(_) => Ok(Response::new(Full::new(Bytes::new()))),
        }
    })
}

fn mock_415_no_accept()
-> impl Service<MockRequest, Response = MockResponse, Error = Infallible, Future: Send> + Clone {
    tower::service_fn(|_req: MockRequest| async {
        let response = Response::builder()
            .status(StatusCode::UNSUPPORTED_MEDIA_TYPE)
            .body(Full::new(Bytes::new()));
        match response {
            Ok(r) => Ok(r),
            Err(_) => Ok(Response::new(Full::new(Bytes::new()))),
        }
    })
}

fn mock_capturing_service() -> (
    impl Service<MockRequest, Response = MockResponse, Error = Infallible, Future: Send> + Clone,
    Arc<std::sync::Mutex<Vec<String>>>,
) {
    let captured = Arc::new(std::sync::Mutex::new(Vec::new()));
    let captured_clone = captured.clone();
    let svc = tower::service_fn(move |req: MockRequest| {
        let captured = captured_clone.clone();
        async move {
            if let Some(ct) = req.headers().get(header::CONTENT_TYPE) {
                if let Ok(s) = ct.to_str() {
                    if let Ok(mut guard) = captured.lock() {
                        guard.push(s.to_string());
                    }
                }
            }
            Ok(Response::new(Full::new(Bytes::new())))
        }
    });
    (svc, captured)
}

fn json_format() -> Arc<dyn ErasedFormat> {
    Arc::new(JsonFormat)
}

fn xml_format() -> Arc<dyn ErasedFormat> {
    Arc::new(XmlFormat)
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
// Format Caching Tests
// =============================================================================

#[tokio::test]
async fn first_request_uses_highest_priority_format() -> TestResult {
    let json = json_format();
    let xml = xml_format();

    let config = ClientConfig::builder()
        .formats([json.clone(), xml])
        .fallback_format(json.clone())
        .build();

    let layer = ClientNegotiateLayer::new(config);
    let (inner, captured) = mock_capturing_service();
    let mut service = layer.layer(inner);

    let req = Request::builder()
        .uri("/")
        .body(Full::new(Bytes::new()))
        .map_err(|e| TestError(e.to_string()))?;

    let _ = service.ready().await?.call(req).await?;

    let content_types = captured.lock()?;
    assert_eq!(content_types.len(), 1);
    assert_eq!(content_types[0], "application/json");
    Ok(())
}

#[tokio::test]
async fn successful_response_caches_format() -> TestResult {
    let json = json_format();

    let config = ClientConfig::builder()
        .formats([json.clone()])
        .fallback_format(json.clone())
        .build();

    let layer = ClientNegotiateLayer::new(config);
    let mut service = layer.layer(mock_ok_service());

    assert!(service.cached_format().is_none());

    let req = Request::builder()
        .uri("/")
        .body(Full::new(Bytes::new()))
        .map_err(|e| TestError(e.to_string()))?;

    let _ = service.ready().await?.call(req).await?;

    let cached = service.cached_format();
    assert!(cached.is_some());
    assert_eq!(
        cached
            .ok_or_else(|| TestError("no cached format".to_string()))?
            .content_type_header(),
        "application/json"
    );
    Ok(())
}

#[tokio::test]
async fn subsequent_requests_use_cached_format() -> TestResult {
    let json = json_format();
    let xml = xml_format();

    let config = ClientConfig::builder()
        .formats([json.clone(), xml.clone()])
        .fallback_format(json.clone())
        .build();

    let layer = ClientNegotiateLayer::new(config);
    let (inner, captured) = mock_capturing_service();
    let mut service = layer.layer(inner);

    for _ in 0..3 {
        let req = Request::builder()
            .uri("/")
            .body(Full::new(Bytes::new()))
            .map_err(|e| TestError(e.to_string()))?;
        let _ = service.ready().await?.call(req).await?;
    }

    let content_types = captured.lock()?;
    assert_eq!(content_types.len(), 3);
    for ct in content_types.iter() {
        assert_eq!(ct, "application/json");
    }
    Ok(())
}

#[tokio::test]
async fn cached_format_returns_stored_format() -> TestResult {
    let json = json_format();

    let config = ClientConfig::builder()
        .formats([json.clone()])
        .fallback_format(json.clone())
        .build();

    let layer = ClientNegotiateLayer::new(config);
    let mut service = layer.layer(mock_ok_service());

    assert!(service.cached_format().is_none());

    let req = Request::builder()
        .uri("/")
        .body(Full::new(Bytes::new()))
        .map_err(|e| TestError(e.to_string()))?;

    let _ = service.ready().await?.call(req).await?;

    let cached = service
        .cached_format()
        .ok_or_else(|| TestError("no cached format".to_string()))?;
    assert_eq!(cached.content_type_header(), "application/json");
    Ok(())
}

// =============================================================================
// 415 Handling with Accept-Post/Accept-Patch
// =============================================================================

#[tokio::test]
async fn parses_accept_post_header_on_415() -> TestResult {
    let json = json_format();
    let xml = xml_format();

    let config = ClientConfig::builder()
        .formats([json.clone(), xml.clone()])
        .fallback_format(json.clone())
        .build();

    let layer = ClientNegotiateLayer::new(config);
    let mut service = layer.layer(mock_415_with_accept_post("application/xml"));

    let req = Request::builder()
        .uri("/")
        .body(Full::new(Bytes::new()))
        .map_err(|e| TestError(e.to_string()))?;

    let response = service.ready().await?.call(req).await?;

    assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);

    let cached = service
        .cached_format()
        .ok_or_else(|| TestError("no cached format".to_string()))?;
    assert_eq!(cached.content_type_header(), "application/xml");
    Ok(())
}

#[tokio::test]
async fn parses_accept_patch_header_on_415() -> TestResult {
    let json = json_format();
    let xml = xml_format();

    let config = ClientConfig::builder()
        .formats([json.clone(), xml.clone()])
        .fallback_format(json.clone())
        .build();

    let layer = ClientNegotiateLayer::new(config);
    let mut service = layer.layer(mock_415_with_accept_patch("application/xml"));

    let req = Request::builder()
        .uri("/")
        .body(Full::new(Bytes::new()))
        .map_err(|e| TestError(e.to_string()))?;

    let response = service.ready().await?.call(req).await?;

    assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);

    let cached = service
        .cached_format()
        .ok_or_else(|| TestError("no cached format".to_string()))?;
    assert_eq!(cached.content_type_header(), "application/xml");
    Ok(())
}

#[tokio::test]
async fn selects_matching_format_from_server_accept_post_list() -> TestResult {
    let json = json_format();
    let xml = xml_format();

    let config = ClientConfig::builder()
        .formats([json.clone(), xml.clone()])
        .fallback_format(xml.clone())
        .build();

    let layer = ClientNegotiateLayer::new(config);
    // Server lists json first in Accept-Post, client supports both
    let mut service = layer.layer(mock_415_with_accept_post(
        "application/json, application/xml",
    ));

    let req = Request::builder()
        .uri("/")
        .body(Full::new(Bytes::new()))
        .map_err(|e| TestError(e.to_string()))?;

    let _ = service.ready().await?.call(req).await?;

    // Selects first matching format from server's Accept-Post header
    let cached = service
        .cached_format()
        .ok_or_else(|| TestError("no cached format".to_string()))?;
    assert_eq!(cached.content_type_header(), "application/json");
    Ok(())
}

#[tokio::test]
async fn caches_negotiated_format_from_415() -> TestResult {
    let json = json_format();
    let xml = xml_format();

    let config = ClientConfig::builder()
        .formats([json.clone(), xml.clone()])
        .fallback_format(json.clone())
        .build();

    let layer = ClientNegotiateLayer::new(config);
    let mut service = layer.layer(mock_415_with_accept_post("application/xml"));

    assert!(service.cached_format().is_none());

    let req = Request::builder()
        .uri("/")
        .body(Full::new(Bytes::new()))
        .map_err(|e| TestError(e.to_string()))?;

    let _ = service.ready().await?.call(req).await?;

    assert!(service.cached_format().is_some());
    let cached = service
        .cached_format()
        .ok_or_else(|| TestError("no cached format".to_string()))?;
    assert_eq!(cached.content_type_header(), "application/xml");
    Ok(())
}

#[tokio::test]
async fn response_415_returned_unchanged() -> TestResult {
    let json = json_format();

    let config = ClientConfig::builder()
        .formats([json.clone()])
        .fallback_format(json.clone())
        .build();

    let layer = ClientNegotiateLayer::new(config);
    let mut service = layer.layer(mock_415_with_accept_post("application/xml"));

    let req = Request::builder()
        .uri("/")
        .body(Full::new(Bytes::new()))
        .map_err(|e| TestError(e.to_string()))?;

    let response = service.ready().await?.call(req).await?;

    assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert!(response.headers().contains_key(ACCEPT_POST));
    Ok(())
}

// =============================================================================
// 415 Without Accept Header
// =============================================================================

#[tokio::test]
async fn uses_fallback_format_when_no_accept_post_header() -> TestResult {
    let json = json_format();
    let xml = xml_format();

    let config = ClientConfig::builder()
        .formats([json.clone(), xml.clone()])
        .fallback_format(xml.clone())
        .build();

    let layer = ClientNegotiateLayer::new(config);
    let mut service = layer.layer(mock_415_no_accept());

    let req = Request::builder()
        .uri("/")
        .body(Full::new(Bytes::new()))
        .map_err(|e| TestError(e.to_string()))?;

    let _ = service.ready().await?.call(req).await?;

    let cached = service
        .cached_format()
        .ok_or_else(|| TestError("no cached format".to_string()))?;
    assert_eq!(cached.content_type_header(), "application/xml");
    Ok(())
}

#[tokio::test]
async fn caches_fallback_format_on_415_without_accept() -> TestResult {
    let json = json_format();
    let xml = xml_format();

    let config = ClientConfig::builder()
        .formats([json.clone()])
        .fallback_format(xml.clone())
        .build();

    let layer = ClientNegotiateLayer::new(config);
    let mut service = layer.layer(mock_415_no_accept());

    assert!(service.cached_format().is_none());

    let req = Request::builder()
        .uri("/")
        .body(Full::new(Bytes::new()))
        .map_err(|e| TestError(e.to_string()))?;

    let _ = service.ready().await?.call(req).await?;

    assert!(service.cached_format().is_some());
    let cached = service
        .cached_format()
        .ok_or_else(|| TestError("no cached format".to_string()))?;
    assert_eq!(cached.content_type_header(), "application/xml");
    Ok(())
}

// =============================================================================
// Per-Request Format Override
// =============================================================================

#[tokio::test]
async fn with_format_bypasses_cache_and_config() -> TestResult {
    let json = json_format();
    let xml = xml_format();

    let config = ClientConfig::builder()
        .formats([json.clone()])
        .fallback_format(json.clone())
        .build();

    let layer = ClientNegotiateLayer::new(config);
    let (inner, captured) = mock_capturing_service();
    let mut service = layer.layer(inner);

    let req = Request::builder()
        .uri("/")
        .body(Full::new(Bytes::new()))
        .map_err(|e| TestError(e.to_string()))?;
    let _ = service.ready().await?.call(req).await?;

    let req_with_override = Request::builder()
        .uri("/")
        .body(Full::new(Bytes::new()))
        .map_err(|e| TestError(e.to_string()))?
        .with_format(xml.clone());

    let _ = service.ready().await?.call(req_with_override).await?;

    let content_types = captured.lock()?;
    assert_eq!(content_types.len(), 2);
    assert_eq!(content_types[0], "application/json");
    assert_eq!(content_types[1], "application/xml");
    Ok(())
}

#[tokio::test]
async fn override_does_not_affect_cache() -> TestResult {
    let json = json_format();
    let xml = xml_format();

    let config = ClientConfig::builder()
        .formats([json.clone()])
        .fallback_format(json.clone())
        .build();

    let layer = ClientNegotiateLayer::new(config);
    let (inner, captured) = mock_capturing_service();
    let mut service = layer.layer(inner);

    let req = Request::builder()
        .uri("/")
        .body(Full::new(Bytes::new()))
        .map_err(|e| TestError(e.to_string()))?;
    let _ = service.ready().await?.call(req).await?;

    let cached_before = service
        .cached_format()
        .ok_or_else(|| TestError("no cached format".to_string()))?;
    assert_eq!(cached_before.content_type_header(), "application/json");

    let req_with_override = Request::builder()
        .uri("/")
        .body(Full::new(Bytes::new()))
        .map_err(|e| TestError(e.to_string()))?
        .with_format(xml.clone());

    let _ = service.ready().await?.call(req_with_override).await?;

    let cached_after = service
        .cached_format()
        .ok_or_else(|| TestError("no cached format".to_string()))?;
    assert_eq!(cached_after.content_type_header(), "application/json");

    let content_types = captured.lock()?;
    assert_eq!(content_types[1], "application/xml");
    Ok(())
}

#[tokio::test]
async fn multiple_requests_can_use_different_overrides() -> TestResult {
    let json = json_format();
    let xml = xml_format();

    let config = ClientConfig::builder()
        .formats([json.clone()])
        .fallback_format(json.clone())
        .build();

    let layer = ClientNegotiateLayer::new(config);
    let (inner, captured) = mock_capturing_service();
    let mut service = layer.layer(inner);

    let req1 = Request::builder()
        .uri("/")
        .body(Full::new(Bytes::new()))
        .map_err(|e| TestError(e.to_string()))?
        .with_format(xml.clone());

    let _ = service.ready().await?.call(req1).await?;

    let req2 = Request::builder()
        .uri("/")
        .body(Full::new(Bytes::new()))
        .map_err(|e| TestError(e.to_string()))?
        .with_format(json.clone());

    let _ = service.ready().await?.call(req2).await?;

    let req3 = Request::builder()
        .uri("/")
        .body(Full::new(Bytes::new()))
        .map_err(|e| TestError(e.to_string()))?
        .with_format(xml.clone());

    let _ = service.ready().await?.call(req3).await?;

    let content_types = captured.lock()?;
    assert_eq!(content_types.len(), 3);
    assert_eq!(content_types[0], "application/xml");
    assert_eq!(content_types[1], "application/json");
    assert_eq!(content_types[2], "application/xml");
    Ok(())
}

// =============================================================================
// Fallback Format Usage
// =============================================================================

#[tokio::test]
async fn empty_formats_list_uses_fallback() -> TestResult {
    let json = json_format();

    let config = ClientConfig::builder()
        .formats([])
        .fallback_format(json.clone())
        .build();

    let layer = ClientNegotiateLayer::new(config);
    let (inner, captured) = mock_capturing_service();
    let mut service = layer.layer(inner);

    let req = Request::builder()
        .uri("/")
        .body(Full::new(Bytes::new()))
        .map_err(|e| TestError(e.to_string()))?;

    let _ = service.ready().await?.call(req).await?;

    let content_types = captured.lock()?;
    assert_eq!(content_types.len(), 1);
    assert_eq!(content_types[0], "application/json");
    Ok(())
}

#[tokio::test]
async fn fallback_used_when_415_has_no_accept_header() -> TestResult {
    let json = json_format();
    let xml = xml_format();

    let config = ClientConfig::builder()
        .formats([json.clone()])
        .fallback_format(xml.clone())
        .build();

    let layer = ClientNegotiateLayer::new(config);
    let mut service = layer.layer(mock_415_no_accept());

    let req = Request::builder()
        .uri("/")
        .body(Full::new(Bytes::new()))
        .map_err(|e| TestError(e.to_string()))?;

    let _ = service.ready().await?.call(req).await?;

    let cached = service
        .cached_format()
        .ok_or_else(|| TestError("no cached format".to_string()))?;
    assert_eq!(cached.content_type_header(), "application/xml");
    Ok(())
}

// =============================================================================
// Accept Header Tests
// =============================================================================

#[tokio::test]
async fn sets_accept_header_on_request() -> TestResult {
    let json = json_format();
    let xml = xml_format();

    let config = ClientConfig::builder()
        .formats([json.clone(), xml.clone()])
        .fallback_format(json.clone())
        .build();

    let layer = ClientNegotiateLayer::new(config);

    let accept_captured = Arc::new(std::sync::Mutex::new(None::<String>));
    let accept_captured_clone = accept_captured.clone();

    let inner = tower::service_fn(move |req: MockRequest| {
        let captured = accept_captured_clone.clone();
        async move {
            if let Some(accept) = req.headers().get(header::ACCEPT) {
                if let Ok(s) = accept.to_str() {
                    if let Ok(mut guard) = captured.lock() {
                        *guard = Some(s.to_string());
                    }
                }
            }
            Ok::<_, Infallible>(Response::new(Full::new(Bytes::new())))
        }
    });

    let mut service = layer.layer(inner);

    let req = Request::builder()
        .uri("/")
        .body(Full::new(Bytes::new()))
        .map_err(|e| TestError(e.to_string()))?;

    let _ = service.ready().await?.call(req).await?;

    let accept = accept_captured.lock()?;
    assert!(accept.is_some());
    let accept_value = accept
        .as_ref()
        .ok_or_else(|| TestError("no accept header".to_string()))?;
    assert!(accept_value.contains("application/json"));
    assert!(accept_value.contains("application/xml"));
    Ok(())
}

// =============================================================================
// Cache Immutability After First Write
// =============================================================================

#[tokio::test]
async fn cache_is_not_overwritten_by_subsequent_responses() -> TestResult {
    let json = json_format();
    let xml = xml_format();

    let config = ClientConfig::builder()
        .formats([json.clone(), xml.clone()])
        .fallback_format(json.clone())
        .build();

    let layer = ClientNegotiateLayer::new(config);
    let mut service = layer.layer(mock_ok_service());

    let req1 = Request::builder()
        .uri("/")
        .body(Full::new(Bytes::new()))
        .map_err(|e| TestError(e.to_string()))?;
    let _ = service.ready().await?.call(req1).await?;

    let cached_after_first = service
        .cached_format()
        .ok_or_else(|| TestError("no cached format".to_string()))?;
    assert_eq!(cached_after_first.content_type_header(), "application/json");

    let req2 = Request::builder()
        .uri("/")
        .body(Full::new(Bytes::new()))
        .map_err(|e| TestError(e.to_string()))?;
    let _ = service.ready().await?.call(req2).await?;

    let cached_after_second = service
        .cached_format()
        .ok_or_else(|| TestError("no cached format".to_string()))?;
    assert_eq!(
        cached_after_second.content_type_header(),
        "application/json"
    );
    Ok(())
}

#[tokio::test]
async fn cache_set_by_success_not_overwritten_by_415() -> TestResult {
    let json = json_format();
    let xml = xml_format();

    let config = ClientConfig::builder()
        .formats([json.clone(), xml.clone()])
        .fallback_format(xml.clone())
        .build();

    let call_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let call_count_clone = call_count.clone();

    let inner = tower::service_fn(move |_req: MockRequest| {
        let count = call_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        async move {
            if count == 0 {
                Ok::<_, Infallible>(Response::new(Full::new(Bytes::new())))
            } else {
                let response = Response::builder()
                    .status(StatusCode::UNSUPPORTED_MEDIA_TYPE)
                    .header(ACCEPT_POST, "application/xml")
                    .body(Full::new(Bytes::new()));
                match response {
                    Ok(r) => Ok(r),
                    Err(_) => Ok(Response::new(Full::new(Bytes::new()))),
                }
            }
        }
    });

    let layer = ClientNegotiateLayer::new(config);
    let mut service = layer.layer(inner);

    let req1 = Request::builder()
        .uri("/")
        .body(Full::new(Bytes::new()))
        .map_err(|e| TestError(e.to_string()))?;
    let _ = service.ready().await?.call(req1).await?;

    let cached = service
        .cached_format()
        .ok_or_else(|| TestError("no cached format".to_string()))?;
    assert_eq!(cached.content_type_header(), "application/json");

    let req2 = Request::builder()
        .uri("/")
        .body(Full::new(Bytes::new()))
        .map_err(|e| TestError(e.to_string()))?;
    let _ = service.ready().await?.call(req2).await?;

    let cached_after = service
        .cached_format()
        .ok_or_else(|| TestError("no cached format".to_string()))?;
    assert_eq!(cached_after.content_type_header(), "application/json");
    Ok(())
}
