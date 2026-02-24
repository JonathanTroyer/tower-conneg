mod common;

use std::sync::Arc;

use bytes::Bytes;
use common::{JsonFormat, XmlFormat};
use http::{Method, Request, Response, StatusCode, header};
use http_body_util::Full;
use tower::{Layer, Service, ServiceExt};
use tower_conneg::{ErasedFormat, NegotiateLayer, NegotiatedFormat, ServerConfig};

fn mock_service() -> impl Service<
    Request<Full<Bytes>>,
    Response = Response<Full<Bytes>>,
    Error = std::convert::Infallible,
    Future = impl std::future::Future<Output = Result<Response<Full<Bytes>>, std::convert::Infallible>>,
> + Clone {
    tower::service_fn(|_req: Request<Full<Bytes>>| async move {
        Ok(Response::new(Full::new(Bytes::new())))
    })
}

fn capturing_service(
    capture: Arc<std::sync::Mutex<Option<NegotiatedFormat>>>,
) -> impl Service<
    Request<Full<Bytes>>,
    Response = Response<Full<Bytes>>,
    Error = std::convert::Infallible,
    Future = impl std::future::Future<Output = Result<Response<Full<Bytes>>, std::convert::Infallible>>,
> + Clone {
    tower::service_fn(move |req: Request<Full<Bytes>>| {
        let capture = Arc::clone(&capture);
        async move {
            if let Some(negotiated) = req.extensions().get::<NegotiatedFormat>() {
                *capture.lock().unwrap() = Some(negotiated.clone());
            }
            Ok(Response::new(Full::new(Bytes::new())))
        }
    })
}

fn json_format() -> Arc<dyn ErasedFormat> {
    Arc::new(JsonFormat)
}

fn xml_format() -> Arc<dyn ErasedFormat> {
    Arc::new(XmlFormat)
}

fn build_config(
    formats: Vec<Arc<dyn ErasedFormat>>,
    fallback: Arc<dyn ErasedFormat>,
) -> ServerConfig {
    ServerConfig::builder()
        .formats(formats)
        .fallback_format(fallback)
        .build()
}

fn build_strict_config(
    formats: Vec<Arc<dyn ErasedFormat>>,
    fallback: Arc<dyn ErasedFormat>,
) -> ServerConfig {
    ServerConfig::builder()
        .formats(formats)
        .fallback_format(fallback)
        .strict(true)
        .build()
}

// Accept header negotiation tests

#[tokio::test]
async fn accept_missing_uses_fallback() {
    let json = json_format();
    let config = build_config(vec![json.clone(), xml_format()], json);
    let layer = NegotiateLayer::new(config);
    let mut service = layer.layer(mock_service());

    let req = Request::builder()
        .uri("/")
        .body(Full::new(Bytes::new()))
        .unwrap();

    let response = service.ready().await.unwrap().call(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn accept_missing_uses_fallback_with_xml_default() {
    let xml = xml_format();
    let config = build_config(vec![json_format(), xml.clone()], xml);
    let capture = Arc::new(std::sync::Mutex::new(None));
    let layer = NegotiateLayer::new(config);
    let mut service = layer.layer(capturing_service(Arc::clone(&capture)));

    let req = Request::builder()
        .uri("/")
        .body(Full::new(Bytes::new()))
        .unwrap();

    let response = service.ready().await.unwrap().call(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let captured = capture.lock().unwrap();
    let negotiated = captured.as_ref().expect("NegotiatedFormat should be set");
    assert_eq!(
        negotiated.response_format().content_type_header(),
        "application/xml"
    );
}

#[tokio::test]
async fn accept_exact_match_json() {
    let json = json_format();
    let config = build_config(vec![json.clone(), xml_format()], json);
    let capture = Arc::new(std::sync::Mutex::new(None));
    let layer = NegotiateLayer::new(config);
    let mut service = layer.layer(capturing_service(Arc::clone(&capture)));

    let req = Request::builder()
        .uri("/")
        .header(header::ACCEPT, "application/json")
        .body(Full::new(Bytes::new()))
        .unwrap();

    let response = service.ready().await.unwrap().call(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let captured = capture.lock().unwrap();
    let negotiated = captured.as_ref().expect("NegotiatedFormat should be set");
    assert_eq!(
        negotiated.response_format().content_type_header(),
        "application/json"
    );
}

#[tokio::test]
async fn accept_exact_match_xml() {
    let json = json_format();
    let config = build_config(vec![json.clone(), xml_format()], json);
    let capture = Arc::new(std::sync::Mutex::new(None));
    let layer = NegotiateLayer::new(config);
    let mut service = layer.layer(capturing_service(Arc::clone(&capture)));

    let req = Request::builder()
        .uri("/")
        .header(header::ACCEPT, "application/xml")
        .body(Full::new(Bytes::new()))
        .unwrap();

    let response = service.ready().await.unwrap().call(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let captured = capture.lock().unwrap();
    let negotiated = captured.as_ref().expect("NegotiatedFormat should be set");
    assert_eq!(
        negotiated.response_format().content_type_header(),
        "application/xml"
    );
}

#[tokio::test]
async fn accept_type_wildcard_application() {
    let json = json_format();
    let config = build_config(vec![json.clone(), xml_format()], json);
    let capture = Arc::new(std::sync::Mutex::new(None));
    let layer = NegotiateLayer::new(config);
    let mut service = layer.layer(capturing_service(Arc::clone(&capture)));

    let req = Request::builder()
        .uri("/")
        .header(header::ACCEPT, "application/*")
        .body(Full::new(Bytes::new()))
        .unwrap();

    let response = service.ready().await.unwrap().call(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let captured = capture.lock().unwrap();
    let negotiated = captured.as_ref().expect("NegotiatedFormat should be set");
    // Type wildcard should match, implementation chooses first matching format
    let content_type = negotiated.response_format().content_type_header();
    assert!(
        content_type == "application/json" || content_type == "application/xml",
        "Expected application/json or application/xml, got {:?}",
        content_type
    );
}

#[tokio::test]
async fn accept_full_wildcard() {
    let json = json_format();
    let config = build_config(vec![json.clone(), xml_format()], json);
    let capture = Arc::new(std::sync::Mutex::new(None));
    let layer = NegotiateLayer::new(config);
    let mut service = layer.layer(capturing_service(Arc::clone(&capture)));

    let req = Request::builder()
        .uri("/")
        .header(header::ACCEPT, "*/*")
        .body(Full::new(Bytes::new()))
        .unwrap();

    let response = service.ready().await.unwrap().call(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let captured = capture.lock().unwrap();
    let negotiated = captured.as_ref().expect("NegotiatedFormat should be set");
    // Full wildcard should use fallback format (json)
    assert_eq!(
        negotiated.response_format().content_type_header(),
        "application/json"
    );
}

#[tokio::test]
async fn accept_quality_values_prefers_higher_quality() {
    let json = json_format();
    let config = build_config(vec![json.clone(), xml_format()], json);
    let capture = Arc::new(std::sync::Mutex::new(None));
    let layer = NegotiateLayer::new(config);
    let mut service = layer.layer(capturing_service(Arc::clone(&capture)));

    // XML has higher quality value, so it should be preferred
    let req = Request::builder()
        .uri("/")
        .header(
            header::ACCEPT,
            "application/json;q=0.5, application/xml;q=0.9",
        )
        .body(Full::new(Bytes::new()))
        .unwrap();

    let response = service.ready().await.unwrap().call(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let captured = capture.lock().unwrap();
    let negotiated = captured.as_ref().expect("NegotiatedFormat should be set");
    assert_eq!(
        negotiated.response_format().content_type_header(),
        "application/xml"
    );
}

#[tokio::test]
async fn accept_quality_values_json_preferred() {
    let json = json_format();
    let config = build_config(vec![json.clone(), xml_format()], json);
    let capture = Arc::new(std::sync::Mutex::new(None));
    let layer = NegotiateLayer::new(config);
    let mut service = layer.layer(capturing_service(Arc::clone(&capture)));

    // JSON has implicit q=1.0, XML has q=0.9
    let req = Request::builder()
        .uri("/")
        .header(header::ACCEPT, "application/xml;q=0.9, application/json")
        .body(Full::new(Bytes::new()))
        .unwrap();

    let response = service.ready().await.unwrap().call(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let captured = capture.lock().unwrap();
    let negotiated = captured.as_ref().expect("NegotiatedFormat should be set");
    assert_eq!(
        negotiated.response_format().content_type_header(),
        "application/json"
    );
}

// Content-Type parsing tests

#[tokio::test]
async fn content_type_missing_results_in_response_only() {
    let json = json_format();
    let config = build_config(vec![json.clone(), xml_format()], json);
    let capture = Arc::new(std::sync::Mutex::new(None));
    let layer = NegotiateLayer::new(config);
    let mut service = layer.layer(capturing_service(Arc::clone(&capture)));

    let req = Request::builder()
        .method(Method::POST)
        .uri("/")
        .header(header::ACCEPT, "application/json")
        .body(Full::new(Bytes::new()))
        .unwrap();

    let response = service.ready().await.unwrap().call(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let captured = capture.lock().unwrap();
    let negotiated = captured.as_ref().expect("NegotiatedFormat should be set");
    assert!(
        negotiated.request_format().is_none(),
        "request_format should be None when Content-Type is missing"
    );
}

#[tokio::test]
async fn content_type_exact_match_json() {
    let json = json_format();
    let config = build_config(vec![json.clone(), xml_format()], json);
    let capture = Arc::new(std::sync::Mutex::new(None));
    let layer = NegotiateLayer::new(config);
    let mut service = layer.layer(capturing_service(Arc::clone(&capture)));

    let req = Request::builder()
        .method(Method::POST)
        .uri("/")
        .header(header::ACCEPT, "application/json")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Full::new(Bytes::new()))
        .unwrap();

    let response = service.ready().await.unwrap().call(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let captured = capture.lock().unwrap();
    let negotiated = captured.as_ref().expect("NegotiatedFormat should be set");
    let request_format = negotiated
        .request_format()
        .expect("request_format should be set");
    assert_eq!(request_format.content_type_header(), "application/json");
}

#[tokio::test]
async fn content_type_exact_match_xml() {
    let json = json_format();
    let config = build_config(vec![json.clone(), xml_format()], json);
    let capture = Arc::new(std::sync::Mutex::new(None));
    let layer = NegotiateLayer::new(config);
    let mut service = layer.layer(capturing_service(Arc::clone(&capture)));

    let req = Request::builder()
        .method(Method::POST)
        .uri("/")
        .header(header::ACCEPT, "application/json")
        .header(header::CONTENT_TYPE, "application/xml")
        .body(Full::new(Bytes::new()))
        .unwrap();

    let response = service.ready().await.unwrap().call(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let captured = capture.lock().unwrap();
    let negotiated = captured.as_ref().expect("NegotiatedFormat should be set");
    let request_format = negotiated
        .request_format()
        .expect("request_format should be set");
    assert_eq!(request_format.content_type_header(), "application/xml");
}

#[tokio::test]
async fn content_type_unsupported_returns_415() {
    let json = json_format();
    let config = build_config(vec![json.clone(), xml_format()], json);
    let layer = NegotiateLayer::new(config);
    let mut service = layer.layer(mock_service());

    let req = Request::builder()
        .method(Method::POST)
        .uri("/")
        .header(header::ACCEPT, "application/json")
        .header(header::CONTENT_TYPE, "text/plain")
        .body(Full::new(Bytes::new()))
        .unwrap();

    let response = service.ready().await.unwrap().call(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
}

// Error response tests

#[tokio::test]
async fn strict_mode_no_accept_match_returns_406() {
    let json = json_format();
    let config = build_strict_config(vec![json.clone(), xml_format()], json);
    let layer = NegotiateLayer::new(config);
    let mut service = layer.layer(mock_service());

    let req = Request::builder()
        .uri("/")
        .header(header::ACCEPT, "text/plain")
        .body(Full::new(Bytes::new()))
        .unwrap();

    let response = service.ready().await.unwrap().call(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_ACCEPTABLE);
}

#[tokio::test]
async fn non_strict_mode_no_accept_match_uses_fallback() {
    let json = json_format();
    let config = build_config(vec![json.clone(), xml_format()], json);
    let capture = Arc::new(std::sync::Mutex::new(None));
    let layer = NegotiateLayer::new(config);
    let mut service = layer.layer(capturing_service(Arc::clone(&capture)));

    let req = Request::builder()
        .uri("/")
        .header(header::ACCEPT, "text/plain")
        .body(Full::new(Bytes::new()))
        .unwrap();

    let response = service.ready().await.unwrap().call(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let captured = capture.lock().unwrap();
    let negotiated = captured.as_ref().expect("NegotiatedFormat should be set");
    assert_eq!(
        negotiated.response_format().content_type_header(),
        "application/json"
    );
}

#[tokio::test]
async fn unsupported_content_type_post_includes_accept_post_header() {
    let json = json_format();
    let config = build_config(vec![json.clone(), xml_format()], json);
    let layer = NegotiateLayer::new(config);
    let mut service = layer.layer(mock_service());

    let req = Request::builder()
        .method(Method::POST)
        .uri("/")
        .header(header::CONTENT_TYPE, "text/plain")
        .body(Full::new(Bytes::new()))
        .unwrap();

    let response = service.ready().await.unwrap().call(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);

    let accept_post = response.headers().get("accept-post");
    assert!(
        accept_post.is_some(),
        "Accept-Post header should be present"
    );
    let value = accept_post.unwrap().to_str().unwrap();
    assert!(
        value.contains("application/json") && value.contains("application/xml"),
        "Accept-Post should list supported formats: {}",
        value
    );
}

#[tokio::test]
async fn unsupported_content_type_patch_includes_accept_patch_header() {
    let json = json_format();
    let config = build_config(vec![json.clone(), xml_format()], json);
    let layer = NegotiateLayer::new(config);
    let mut service = layer.layer(mock_service());

    let req = Request::builder()
        .method(Method::PATCH)
        .uri("/")
        .header(header::CONTENT_TYPE, "text/plain")
        .body(Full::new(Bytes::new()))
        .unwrap();

    let response = service.ready().await.unwrap().call(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);

    let accept_patch = response.headers().get("accept-patch");
    assert!(
        accept_patch.is_some(),
        "Accept-Patch header should be present"
    );
    let value = accept_patch.unwrap().to_str().unwrap();
    assert!(
        value.contains("application/json") && value.contains("application/xml"),
        "Accept-Patch should list supported formats: {}",
        value
    );
}

#[tokio::test]
async fn unsupported_content_type_get_no_accept_header() {
    let json = json_format();
    let config = build_config(vec![json.clone(), xml_format()], json);
    let layer = NegotiateLayer::new(config);
    let mut service = layer.layer(mock_service());

    let req = Request::builder()
        .method(Method::GET)
        .uri("/")
        .header(header::CONTENT_TYPE, "text/plain")
        .body(Full::new(Bytes::new()))
        .unwrap();

    let response = service.ready().await.unwrap().call(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);

    assert!(
        response.headers().get("accept-post").is_none(),
        "Accept-Post header should not be present for GET"
    );
    assert!(
        response.headers().get("accept-patch").is_none(),
        "Accept-Patch header should not be present for GET"
    );
}

#[tokio::test]
async fn unsupported_content_type_put_no_accept_header() {
    let json = json_format();
    let config = build_config(vec![json.clone(), xml_format()], json);
    let layer = NegotiateLayer::new(config);
    let mut service = layer.layer(mock_service());

    let req = Request::builder()
        .method(Method::PUT)
        .uri("/")
        .header(header::CONTENT_TYPE, "text/plain")
        .body(Full::new(Bytes::new()))
        .unwrap();

    let response = service.ready().await.unwrap().call(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);

    assert!(
        response.headers().get("accept-post").is_none(),
        "Accept-Post header should not be present for PUT"
    );
    assert!(
        response.headers().get("accept-patch").is_none(),
        "Accept-Patch header should not be present for PUT"
    );
}

// NegotiatedFormat in extensions tests

#[tokio::test]
async fn negotiated_format_stored_in_extensions() {
    let json = json_format();
    let config = build_config(vec![json.clone(), xml_format()], json);
    let capture = Arc::new(std::sync::Mutex::new(None));
    let layer = NegotiateLayer::new(config);
    let mut service = layer.layer(capturing_service(Arc::clone(&capture)));

    let req = Request::builder()
        .method(Method::POST)
        .uri("/")
        .header(header::ACCEPT, "application/xml")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Full::new(Bytes::new()))
        .unwrap();

    let response = service.ready().await.unwrap().call(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let captured = capture.lock().unwrap();
    assert!(
        captured.is_some(),
        "NegotiatedFormat should be stored in extensions"
    );
}

#[tokio::test]
async fn negotiated_format_response_and_request_formats_differ() {
    let json = json_format();
    let config = build_config(vec![json.clone(), xml_format()], json);
    let capture = Arc::new(std::sync::Mutex::new(None));
    let layer = NegotiateLayer::new(config);
    let mut service = layer.layer(capturing_service(Arc::clone(&capture)));

    // Request with XML body but asking for JSON response
    let req = Request::builder()
        .method(Method::POST)
        .uri("/")
        .header(header::ACCEPT, "application/json")
        .header(header::CONTENT_TYPE, "application/xml")
        .body(Full::new(Bytes::new()))
        .unwrap();

    let response = service.ready().await.unwrap().call(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let captured = capture.lock().unwrap();
    let negotiated = captured.as_ref().expect("NegotiatedFormat should be set");
    assert_eq!(
        negotiated.response_format().content_type_header(),
        "application/json"
    );
    assert_eq!(
        negotiated
            .request_format()
            .expect("request_format should be set")
            .content_type_header(),
        "application/xml"
    );
}

#[tokio::test]
async fn negotiated_format_response_and_request_formats_same() {
    let json = json_format();
    let config = build_config(vec![json.clone(), xml_format()], json);
    let capture = Arc::new(std::sync::Mutex::new(None));
    let layer = NegotiateLayer::new(config);
    let mut service = layer.layer(capturing_service(Arc::clone(&capture)));

    let req = Request::builder()
        .method(Method::POST)
        .uri("/")
        .header(header::ACCEPT, "application/json")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Full::new(Bytes::new()))
        .unwrap();

    let response = service.ready().await.unwrap().call(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let captured = capture.lock().unwrap();
    let negotiated = captured.as_ref().expect("NegotiatedFormat should be set");
    assert_eq!(
        negotiated.response_format().content_type_header(),
        "application/json"
    );
    assert_eq!(
        negotiated
            .request_format()
            .expect("request_format should be set")
            .content_type_header(),
        "application/json"
    );
}

// Edge cases

#[tokio::test]
async fn single_format_config() {
    let json = json_format();
    let config = build_config(vec![json.clone()], json);
    let capture = Arc::new(std::sync::Mutex::new(None));
    let layer = NegotiateLayer::new(config);
    let mut service = layer.layer(capturing_service(Arc::clone(&capture)));

    let req = Request::builder()
        .uri("/")
        .header(header::ACCEPT, "application/json")
        .body(Full::new(Bytes::new()))
        .unwrap();

    let response = service.ready().await.unwrap().call(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let captured = capture.lock().unwrap();
    let negotiated = captured.as_ref().expect("NegotiatedFormat should be set");
    assert_eq!(
        negotiated.response_format().content_type_header(),
        "application/json"
    );
}

#[tokio::test]
async fn empty_formats_uses_fallback() {
    let json = json_format();
    let config = ServerConfig::builder()
        .formats(std::iter::empty::<Arc<dyn ErasedFormat>>())
        .fallback_format(json)
        .build();
    let capture = Arc::new(std::sync::Mutex::new(None));
    let layer = NegotiateLayer::new(config);
    let mut service = layer.layer(capturing_service(Arc::clone(&capture)));

    let req = Request::builder()
        .uri("/")
        .body(Full::new(Bytes::new()))
        .unwrap();

    let response = service.ready().await.unwrap().call(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let captured = capture.lock().unwrap();
    let negotiated = captured.as_ref().expect("NegotiatedFormat should be set");
    assert_eq!(
        negotiated.response_format().content_type_header(),
        "application/json"
    );
}

#[tokio::test]
async fn content_type_with_charset_matches() {
    let json = json_format();
    let config = build_config(vec![json.clone(), xml_format()], json);
    let capture = Arc::new(std::sync::Mutex::new(None));
    let layer = NegotiateLayer::new(config);
    let mut service = layer.layer(capturing_service(Arc::clone(&capture)));

    let req = Request::builder()
        .method(Method::POST)
        .uri("/")
        .header(header::ACCEPT, "application/json")
        .header(header::CONTENT_TYPE, "application/json; charset=utf-8")
        .body(Full::new(Bytes::new()))
        .unwrap();

    let response = service.ready().await.unwrap().call(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let captured = capture.lock().unwrap();
    let negotiated = captured.as_ref().expect("NegotiatedFormat should be set");
    let request_format = negotiated
        .request_format()
        .expect("request_format should be set");
    assert_eq!(request_format.content_type_header(), "application/json");
}

#[tokio::test]
async fn multiple_accept_values_first_match_wins() {
    let json = json_format();
    let config = build_config(vec![json.clone(), xml_format()], json);
    let capture = Arc::new(std::sync::Mutex::new(None));
    let layer = NegotiateLayer::new(config);
    let mut service = layer.layer(capturing_service(Arc::clone(&capture)));

    // Both are q=1.0 (default), but JSON comes first in Accept
    let req = Request::builder()
        .uri("/")
        .header(header::ACCEPT, "application/json, application/xml")
        .body(Full::new(Bytes::new()))
        .unwrap();

    let response = service.ready().await.unwrap().call(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let captured = capture.lock().unwrap();
    let negotiated = captured.as_ref().expect("NegotiatedFormat should be set");
    assert_eq!(
        negotiated.response_format().content_type_header(),
        "application/json"
    );
}
