#![cfg(all(feature = "json", feature = "xml"))]

mod common;

use std::convert::Infallible;
use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use bytes::Bytes;
use http::{Request, Response, StatusCode};
use http_body_util::Full;
use tower::{Service, ServiceExt};
use tower_conneg::{ClientConfig, ErasedFormat, Retry415Helper, RetryError};

use common::{JsonFormat, XmlFormat};

const ACCEPT_POST: http::HeaderName = http::HeaderName::from_static("accept-post");
const ACCEPT_PATCH: http::HeaderName = http::HeaderName::from_static("accept-patch");

type MockRequest = Request<Full<Bytes>>;
type MockResponse = Response<Full<Bytes>>;

fn json_format() -> Arc<dyn ErasedFormat> {
    Arc::new(JsonFormat)
}

fn xml_format() -> Arc<dyn ErasedFormat> {
    Arc::new(XmlFormat)
}

fn mock_ok_service()
-> impl Service<MockRequest, Response = MockResponse, Error = Infallible, Future: Send> + Clone {
    tower::service_fn(|_req: MockRequest| async { Ok(Response::new(Full::new(Bytes::new()))) })
}

fn mock_always_415(
    accept_post: Option<&'static str>,
) -> impl Service<MockRequest, Response = MockResponse, Error = Infallible, Future: Send> + Clone {
    tower::service_fn(move |_req: MockRequest| async move {
        let mut builder = Response::builder().status(StatusCode::UNSUPPORTED_MEDIA_TYPE);
        if let Some(header) = accept_post {
            builder = builder.header(ACCEPT_POST, header);
        }
        match builder.body(Full::new(Bytes::new())) {
            Ok(r) => Ok(r),
            Err(_) => Ok(Response::new(Full::new(Bytes::new()))),
        }
    })
}

fn mock_415_then_ok(
    accept_post: &'static str,
) -> (
    impl Service<MockRequest, Response = MockResponse, Error = Infallible, Future: Send> + Clone,
    Arc<AtomicUsize>,
) {
    let call_count = Arc::new(AtomicUsize::new(0));
    let call_count_clone = call_count.clone();

    let svc = tower::service_fn(move |_req: MockRequest| {
        let count = call_count_clone.fetch_add(1, Ordering::SeqCst);
        async move {
            if count == 0 {
                let response = Response::builder()
                    .status(StatusCode::UNSUPPORTED_MEDIA_TYPE)
                    .header(ACCEPT_POST, accept_post)
                    .body(Full::new(Bytes::new()));
                match response {
                    Ok(r) => Ok(r),
                    Err(_) => Ok(Response::new(Full::new(Bytes::new()))),
                }
            } else {
                Ok::<_, Infallible>(Response::new(Full::new(Bytes::new())))
            }
        }
    });

    (svc, call_count)
}

fn mock_415_then_ok_with_accept_patch(
    accept_patch: &'static str,
) -> (
    impl Service<MockRequest, Response = MockResponse, Error = Infallible, Future: Send> + Clone,
    Arc<AtomicUsize>,
) {
    let call_count = Arc::new(AtomicUsize::new(0));
    let call_count_clone = call_count.clone();

    let svc = tower::service_fn(move |_req: MockRequest| {
        let count = call_count_clone.fetch_add(1, Ordering::SeqCst);
        async move {
            if count == 0 {
                let response = Response::builder()
                    .status(StatusCode::UNSUPPORTED_MEDIA_TYPE)
                    .header(ACCEPT_PATCH, accept_patch)
                    .body(Full::new(Bytes::new()));
                match response {
                    Ok(r) => Ok(r),
                    Err(_) => Ok(Response::new(Full::new(Bytes::new()))),
                }
            } else {
                Ok::<_, Infallible>(Response::new(Full::new(Bytes::new())))
            }
        }
    });

    (svc, call_count)
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

impl From<RetryError<Infallible>> for TestError {
    fn from(e: RetryError<Infallible>) -> Self {
        TestError(e.to_string())
    }
}

type TestResult = Result<(), TestError>;

// =============================================================================
// Success on First Try
// =============================================================================

#[tokio::test]
async fn success_on_first_try_returns_immediately() -> TestResult {
    let json = json_format();

    let config = ClientConfig::builder()
        .formats([json.clone()])
        .fallback_format(json.clone())
        .build();

    let helper = Retry415Helper::new(config, 3);
    let service = mock_ok_service();

    let response = helper
        .call(service, |_format| {
            Request::builder()
                .uri("/")
                .body(Full::new(Bytes::new()))
                .expect("request builder")
        })
        .await?;

    assert!(response.status().is_success());
    Ok(())
}

// =============================================================================
// Retry on 415
// =============================================================================

#[tokio::test]
async fn retries_with_format_from_accept_post() -> TestResult {
    let json = json_format();
    let xml = xml_format();

    let config = ClientConfig::builder()
        .formats([json.clone(), xml.clone()])
        .fallback_format(json.clone())
        .build();

    let helper = Retry415Helper::new(config, 3);
    let (service, call_count) = mock_415_then_ok("application/xml");

    let formats_used = Arc::new(std::sync::Mutex::new(Vec::new()));
    let formats_used_clone = formats_used.clone();

    let response = helper
        .call(service, move |format| {
            if let Ok(mut guard) = formats_used_clone.lock() {
                guard.push(
                    format
                        .content_type_header()
                        .to_str()
                        .unwrap_or("")
                        .to_string(),
                );
            }
            Request::builder()
                .uri("/")
                .body(Full::new(Bytes::new()))
                .expect("request builder")
        })
        .await?;

    assert!(response.status().is_success());
    assert_eq!(call_count.load(Ordering::SeqCst), 2);

    let formats = formats_used.lock().map_err(|e| TestError(e.to_string()))?;
    assert_eq!(formats.len(), 2);
    assert_eq!(formats[0], "application/json");
    assert_eq!(formats[1], "application/xml");

    Ok(())
}

#[tokio::test]
async fn retries_with_format_from_accept_patch() -> TestResult {
    let json = json_format();
    let xml = xml_format();

    let config = ClientConfig::builder()
        .formats([json.clone(), xml.clone()])
        .fallback_format(json.clone())
        .build();

    let helper = Retry415Helper::new(config, 3);
    let (service, call_count) = mock_415_then_ok_with_accept_patch("application/xml");

    let response = helper
        .call(service, |_format| {
            Request::builder()
                .uri("/")
                .body(Full::new(Bytes::new()))
                .expect("request builder")
        })
        .await?;

    assert!(response.status().is_success());
    assert_eq!(call_count.load(Ordering::SeqCst), 2);

    Ok(())
}

// =============================================================================
// Max Attempts Limit
// =============================================================================

#[tokio::test]
async fn respects_max_attempts_limit() -> TestResult {
    let json = json_format();
    let xml = xml_format();

    let config = ClientConfig::builder()
        .formats([json.clone(), xml.clone()])
        .fallback_format(json.clone())
        .build();

    let helper = Retry415Helper::new(config, 2);

    let call_count = Arc::new(AtomicUsize::new(0));
    let call_count_clone = call_count.clone();

    let counting_service = tower::service_fn(move |req: MockRequest| {
        call_count_clone.fetch_add(1, Ordering::SeqCst);
        let mut inner = mock_always_415(Some("application/xml"));
        async move { inner.ready().await?.call(req).await }
    });

    let response = helper
        .call(counting_service, |_format| {
            Request::builder()
                .uri("/")
                .body(Full::new(Bytes::new()))
                .expect("request builder")
        })
        .await?;

    assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert_eq!(call_count.load(Ordering::SeqCst), 2);

    Ok(())
}

#[tokio::test]
async fn max_attempts_one_returns_415_immediately() -> TestResult {
    let json = json_format();

    let config = ClientConfig::builder()
        .formats([json.clone()])
        .fallback_format(json.clone())
        .build();

    let helper = Retry415Helper::new(config, 1);
    let (service, call_count) = mock_415_then_ok("application/json");

    let response = helper
        .call(service, |_format| {
            Request::builder()
                .uri("/")
                .body(Full::new(Bytes::new()))
                .expect("request builder")
        })
        .await?;

    assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert_eq!(call_count.load(Ordering::SeqCst), 1);

    Ok(())
}

// =============================================================================
// Fallback Format
// =============================================================================

#[tokio::test]
async fn uses_fallback_when_no_accept_header() -> TestResult {
    let json = json_format();
    let xml = xml_format();

    let config = ClientConfig::builder()
        .formats([json.clone()])
        .fallback_format(xml.clone())
        .build();

    let helper = Retry415Helper::new(config, 3);

    let formats_used = Arc::new(std::sync::Mutex::new(Vec::new()));
    let formats_used_clone = formats_used.clone();

    let call_count = Arc::new(AtomicUsize::new(0));
    let call_count_clone = call_count.clone();

    let service = tower::service_fn(move |_req: MockRequest| {
        let count = call_count_clone.fetch_add(1, Ordering::SeqCst);
        async move {
            if count == 0 {
                // First call: 415 with no Accept header
                let response = Response::builder()
                    .status(StatusCode::UNSUPPORTED_MEDIA_TYPE)
                    .body(Full::new(Bytes::new()));
                match response {
                    Ok(r) => Ok(r),
                    Err(_) => Ok(Response::new(Full::new(Bytes::new()))),
                }
            } else {
                Ok::<_, Infallible>(Response::new(Full::new(Bytes::new())))
            }
        }
    });

    let response = helper
        .call(service, move |format| {
            if let Ok(mut guard) = formats_used_clone.lock() {
                guard.push(
                    format
                        .content_type_header()
                        .to_str()
                        .unwrap_or("")
                        .to_string(),
                );
            }
            Request::builder()
                .uri("/")
                .body(Full::new(Bytes::new()))
                .expect("request builder")
        })
        .await?;

    assert!(response.status().is_success());

    let formats = formats_used.lock().map_err(|e| TestError(e.to_string()))?;
    assert_eq!(formats.len(), 2);
    assert_eq!(formats[0], "application/json");
    assert_eq!(formats[1], "application/xml");

    Ok(())
}

// =============================================================================
// Error Display
// =============================================================================

#[test]
fn retry_error_display() {
    let error: RetryError<std::io::Error> =
        RetryError::Service(std::io::Error::new(std::io::ErrorKind::Other, "test error"));
    assert!(error.to_string().contains("service error"));
    assert!(error.to_string().contains("test error"));
}

#[test]
fn retry_error_source() {
    let inner = std::io::Error::new(std::io::ErrorKind::Other, "inner error");
    let error: RetryError<std::io::Error> = RetryError::Service(inner);
    assert!(error.source().is_some());
}
