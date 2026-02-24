//! Client-side content negotiation example.
//!
//! This example demonstrates how to use tower-conneg with an HTTP client to
//! automatically handle content type negotiation for requests and responses.
//!
//! Run with: cargo run --example client --features "json xml"

#![allow(clippy::print_stdout)]
#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::too_many_lines)]

use std::convert::Infallible;
use std::sync::Arc;

use bytes::Bytes;
use http::{Request, Response, StatusCode};
use http_body_util::Full;
use tower::layer::Layer;
use tower::{Service, ServiceExt};
use tower_conneg::{
    ClientConfig, ClientNegotiateLayer, ClientRequestExt, ErasedFormat, JsonFormat, XmlFormat,
};

// ----------------------------------------------------------------------------
// Type alias for clarity
// ----------------------------------------------------------------------------

type MockRequest = Request<Full<Bytes>>;

#[tokio::main]
async fn main() {
    // ========================================================================
    // Step 1: Create format instances
    // ========================================================================
    //
    // Formats define how to serialize/deserialize data. They must be wrapped
    // in Arc for sharing across the middleware stack.

    let json: Arc<dyn ErasedFormat> = Arc::new(JsonFormat);
    let xml: Arc<dyn ErasedFormat> = Arc::new(XmlFormat);

    // ========================================================================
    // Step 2: Configure the client with format priority
    // ========================================================================
    //
    // ClientConfig defines:
    // - `formats`: Priority-ordered list of formats to try (first = highest priority)
    // - `fallback_format`: Used when server returns 415 without Accept-Post header
    //
    // The client will:
    // 1. Use cached format if available (from previous successful request)
    // 2. Otherwise try the highest-priority format
    // 3. On success: cache that format for future requests
    // 4. On 415: parse Accept-Post header and cache the server's preferred format

    let config = ClientConfig::builder()
        .formats([json.clone(), xml.clone()]) // JSON preferred, XML as fallback
        .fallback_format(json.clone()) // Use JSON if server doesn't specify
        .build();

    // ========================================================================
    // Step 3: Wrap the HTTP client with content negotiation
    // ========================================================================
    //
    // ClientNegotiateLayer wraps any Tower service implementing the HTTP Service
    // trait. In a real application, this would be hyper-util's Client:
    //
    // ```
    // use hyper_util::client::legacy::Client;
    // use hyper_util::rt::TokioExecutor;
    // use tower_conneg::HyperClientExt;
    //
    // let client = Client::builder(TokioExecutor::new())
    //     .build_http()
    //     .with_content_negotiation(config);
    // ```

    let layer = ClientNegotiateLayer::new(config);

    // Mock server that simulates server responses for demonstration purposes.
    // In a real application, this would be replaced with hyper-util's Client.
    let mock_server = tower::service_fn(|req: MockRequest| async move {
        // Echo back the Content-Type as our response type
        let content_type = req
            .headers()
            .get(http::header::CONTENT_TYPE)
            .cloned()
            .unwrap_or_else(|| http::HeaderValue::from_static("application/json"));

        let body = Full::new(Bytes::from(
            r#"{"id":1,"name":"Alice","email":"alice@example.com"}"#,
        ));

        let response = Response::builder()
            .status(StatusCode::OK)
            .header(http::header::CONTENT_TYPE, content_type)
            .body(body)
            .unwrap_or_else(|_| Response::new(Full::new(Bytes::new())));

        Ok::<_, Infallible>(response)
    });

    let mut client = layer.layer(mock_server);

    // ========================================================================
    // Step 4: Make requests - format is automatically applied
    // ========================================================================
    //
    // When making requests:
    // - Content-Type header is automatically set based on the selected format
    // - Accept header is set to advertise all supported formats with q-values
    // - The middleware handles 415 responses and caches the negotiated format

    println!("=== Basic Request ===");

    // Before any requests, no format is cached
    assert!(client.cached_format().is_none());
    println!("Cached format before first request: None");

    let request = Request::builder()
        .method("POST")
        .uri("https://api.example.com/users")
        .body(Full::new(Bytes::new()))
        .expect("request should build");

    let response = client
        .ready()
        .await
        .expect("service should be ready")
        .call(request)
        .await
        .expect("request should succeed");

    println!("Response status: {}", response.status());

    // ========================================================================
    // Step 5: Format caching behavior
    // ========================================================================
    //
    // After a successful request, the format is cached. All subsequent requests
    // from this client will use the cached format automatically.

    println!("\n=== Format Caching ===");

    let cached = client.cached_format().expect("format should be cached");
    println!(
        "Cached format after first request: {}",
        cached.content_type_header().to_str().unwrap_or("unknown")
    );

    // Make another request - it will automatically use the cached format
    let request2 = Request::builder()
        .method("POST")
        .uri("https://api.example.com/users")
        .body(Full::new(Bytes::new()))
        .expect("request should build");

    let _ = client
        .ready()
        .await
        .expect("service should be ready")
        .call(request2)
        .await
        .expect("request should succeed");

    // The cached format remains the same
    let still_cached = client
        .cached_format()
        .expect("format should still be cached");
    assert_eq!(
        cached.content_type_header(),
        still_cached.content_type_header()
    );
    println!("Format remains cached for subsequent requests");

    // ========================================================================
    // Step 6: Per-request format override with .with_format()
    // ========================================================================
    //
    // Sometimes you need to use a specific format for a particular request,
    // ignoring the cache. Use the `.with_format()` extension method:

    println!("\n=== Per-Request Format Override ===");

    let request_override = Request::builder()
        .method("POST")
        .uri("https://api.example.com/users")
        .body(Full::new(Bytes::new()))
        .expect("request should build")
        .with_format(xml.clone()); // Force XML for this request only

    let response_xml = client
        .ready()
        .await
        .expect("service should be ready")
        .call(request_override)
        .await
        .expect("request should succeed");

    let xml_content_type = response_xml
        .headers()
        .get(http::header::CONTENT_TYPE)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");
    println!("Override request used: {xml_content_type}");

    // The cache is NOT affected by override requests
    let after_override = client
        .cached_format()
        .expect("format should still be cached");
    assert_eq!(
        cached.content_type_header(),
        after_override.content_type_header()
    );
    println!("Cache unchanged after override request");

    // ========================================================================
    // Step 7: Handling 415 Unsupported Media Type
    // ========================================================================
    //
    // When a server returns 415, the middleware:
    // 1. Parses the Accept-Post or Accept-Patch header
    // 2. Selects the highest-priority client format that the server supports
    // 3. Caches that format for future requests
    // 4. Returns the 415 response (no auto-retry - that's up to the caller)

    println!("\n=== 415 Handling ===");

    // Create a new client to demonstrate 415 handling
    let config_415 = ClientConfig::builder()
        .formats([json.clone(), xml.clone()])
        .fallback_format(json.clone())
        .build();

    // Mock server that returns 415 Unsupported Media Type with Accept-Post header
    let mock_server_415 = tower::service_fn(|_req: MockRequest| async move {
        let response = Response::builder()
            .status(StatusCode::UNSUPPORTED_MEDIA_TYPE)
            .header("accept-post", "application/xml")
            .body(Full::new(Bytes::new()))
            .unwrap_or_else(|_| Response::new(Full::new(Bytes::new())));

        Ok::<_, Infallible>(response)
    });

    let mut client_415 = ClientNegotiateLayer::new(config_415).layer(mock_server_415);

    let request_415 = Request::builder()
        .method("POST")
        .uri("https://api.example.com/users")
        .body(Full::new(Bytes::new()))
        .expect("request should build");

    let response_415 = client_415
        .ready()
        .await
        .expect("service should be ready")
        .call(request_415)
        .await
        .expect("request should succeed");

    println!("Response status: {}", response_415.status());

    // Even though we got 415, the middleware parsed Accept-Post and cached XML
    let cached_after_415 = client_415
        .cached_format()
        .expect("format should be cached from 415");
    println!(
        "Cached format after 415: {} (from server's Accept-Post)",
        cached_after_415
            .content_type_header()
            .to_str()
            .unwrap_or("unknown")
    );

    // ========================================================================
    // Summary
    // ========================================================================

    println!("\n=== Summary ===");
    println!("1. Configure ClientConfig with formats in priority order");
    println!("2. Wrap your HTTP client with ClientNegotiateLayer");
    println!("3. Make requests - Content-Type and Accept headers are automatic");
    println!("4. The format is cached after the first successful request");
    println!("5. Use .with_format() to override for specific requests");
    println!("6. On 415, the server's preferred format is cached for next time");
}
