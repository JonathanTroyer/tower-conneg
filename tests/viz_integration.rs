#![cfg(all(feature = "viz", feature = "json"))]
#![allow(missing_docs)]

use std::sync::Arc;

use http::header;
use serde::{Deserialize, Serialize};
use viz::{
    Body, FromRequest, Handler, IntoResponse, Request, Response, Result, Router, Tree,
    types::RouteInfo,
};

use tower_conneg::{ErasedFormat, JsonFormat, Negotiate, NegotiateResponse, NegotiatedFormat};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestData {
    value: String,
}

/// A Transform that inserts NegotiatedFormat into request extensions.
#[derive(Clone)]
struct NegotiateTransform;

impl<H: Clone> viz::Transform<H> for NegotiateTransform {
    type Output = NegotiateHandler<H>;

    fn transform(&self, h: H) -> Self::Output {
        NegotiateHandler { inner: h }
    }
}

#[derive(Clone)]
struct NegotiateHandler<H> {
    inner: H,
}

#[viz::async_trait]
impl<H> Handler<Request> for NegotiateHandler<H>
where
    H: Handler<Request, Output = Result<Response>> + Clone,
{
    type Output = Result<Response>;

    async fn call(&self, mut req: Request) -> Self::Output {
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
        self.inner.call(req).await
    }
}

async fn get_handler(mut req: Request) -> Result<Response> {
    let neg = match Negotiate::<()>::extract(&mut req).await {
        Ok(neg) => neg,
        Err(e) => return Ok(e.into_response()),
    };
    Ok(neg
        .respond(TestData {
            value: "hello".to_string(),
        })
        .into_response())
}

async fn post_handler(mut req: Request) -> Result<Response> {
    let neg = match Negotiate::<TestData>::extract(&mut req).await {
        Ok(neg) => neg,
        Err(e) => return Ok(e.into_response()),
    };
    let format = Arc::clone(neg.format());
    let mut data = neg.into_inner();
    data.value = format!("received: {}", data.value);
    Ok(NegotiateResponse::new(data, format).into_response())
}

fn build_router_with_middleware() -> Tree {
    Router::new()
        .get("/", get_handler)
        .post("/", post_handler)
        .with(NegotiateTransform)
        .into()
}

fn build_router_without_middleware() -> Tree {
    Router::new().get("/", get_handler).into()
}

async fn call_handler(
    tree: &Tree,
    method: http::Method,
    path: &str,
    headers: Vec<(&str, &str)>,
    body: Option<Vec<u8>>,
) -> Response {
    let mut builder = http::Request::builder().method(method.clone()).uri(path);

    for (name, value) in headers {
        builder = builder.header(name, value);
    }

    let body = match body {
        Some(b) => Body::Full(b.into()),
        None => Body::Empty,
    };

    let mut req: Request = builder.body(body).expect("valid request");

    let node = tree.find(&method, path);
    match node {
        Some((handler, route)) => {
            req.extensions_mut().insert(Arc::from(RouteInfo {
                id: *route.id,
                pattern: route.pattern(),
                params: route.params().into(),
            }));
            handler
                .call(req)
                .await
                .unwrap_or_else(|e| e.into_response())
        }
        None => {
            let mut resp = Response::default();
            *resp.status_mut() = http::StatusCode::NOT_FOUND;
            resp
        }
    }
}

async fn read_body(response: Response) -> Vec<u8> {
    use http_body_util::BodyExt;
    response
        .into_body()
        .collect()
        .await
        .map(|c| c.to_bytes().to_vec())
        .unwrap_or_default()
}

#[tokio::test]
async fn negotiate_unit_extraction() {
    let tree = build_router_with_middleware();

    let response = call_handler(
        &tree,
        http::Method::GET,
        "/",
        vec![("Accept", "application/json")],
        None,
    )
    .await;

    assert_eq!(response.status(), http::StatusCode::OK);

    let content_type = response.headers().get(header::CONTENT_TYPE);
    assert!(content_type.is_some());
    assert_eq!(content_type.unwrap(), "application/json");
}

#[tokio::test]
async fn negotiate_body_extraction() {
    let tree = build_router_with_middleware();

    let input = TestData {
        value: "test".to_string(),
    };
    let body = serde_json::to_vec(&input).expect("serialize");

    let response = call_handler(
        &tree,
        http::Method::POST,
        "/",
        vec![
            ("Accept", "application/json"),
            ("Content-Type", "application/json"),
        ],
        Some(body),
    )
    .await;

    assert_eq!(response.status(), http::StatusCode::OK);

    let body = read_body(response).await;
    let data: TestData = serde_json::from_slice(&body).expect("deserialize");
    assert_eq!(data.value, "received: test");
}

#[tokio::test]
async fn negotiate_response_serialization() {
    let tree = build_router_with_middleware();

    let response = call_handler(
        &tree,
        http::Method::GET,
        "/",
        vec![("Accept", "application/json")],
        None,
    )
    .await;

    assert_eq!(response.status(), http::StatusCode::OK);

    let content_type = response.headers().get(header::CONTENT_TYPE);
    assert!(content_type.is_some());
    assert_eq!(content_type.unwrap(), "application/json");

    let body = read_body(response).await;
    let data: TestData = serde_json::from_slice(&body).expect("deserialize");
    assert_eq!(data.value, "hello");
}

#[tokio::test]
async fn roundtrip_json() {
    let tree = build_router_with_middleware();

    let input = TestData {
        value: "roundtrip test".to_string(),
    };
    let body = serde_json::to_vec(&input).expect("serialize");

    let response = call_handler(
        &tree,
        http::Method::POST,
        "/",
        vec![
            ("Accept", "application/json"),
            ("Content-Type", "application/json"),
        ],
        Some(body),
    )
    .await;

    assert_eq!(response.status(), http::StatusCode::OK);

    let content_type = response.headers().get(header::CONTENT_TYPE);
    assert!(content_type.is_some());
    assert_eq!(content_type.unwrap(), "application/json");

    let body = read_body(response).await;
    let output: TestData = serde_json::from_slice(&body).expect("deserialize");
    assert_eq!(output.value, "received: roundtrip test");
}

#[tokio::test]
async fn missing_middleware_error() {
    let tree = build_router_without_middleware();

    let response = call_handler(
        &tree,
        http::Method::GET,
        "/",
        vec![("Accept", "application/json")],
        None,
    )
    .await;

    assert_eq!(response.status(), http::StatusCode::NOT_ACCEPTABLE);
}
