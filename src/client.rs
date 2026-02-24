//! Client-side content negotiation Tower middleware.
//!
//! This middleware handles content negotiation for HTTP clients:
//! - Serializes request bodies using the negotiated format
//! - Sets the `Content-Type` header on requests
//! - Parses the response `Content-Type` header
//! - Caches successful format selections for subsequent requests
//!
//! Format selection follows a caching strategy per DESIGN.md:
//! 1. Use cached format if available
//! 2. Otherwise use highest-priority format from config
//! 3. On 2xx success: cache that format
//! 4. On 415 failure: parse Accept-Post/Accept-Patch, cache appropriate format

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, OnceLock};
use std::task::{Context, Poll};

use bytes::Bytes;
use http::{Request, Response, StatusCode, header};
use http_body::Body;
use http_body_util::Full;
use pin_project_lite::pin_project;
use tower::{Layer, Service};

use crate::accept::parse_accept_erased;
use crate::config::ClientConfig;
use crate::format::ErasedFormat;
use crate::{ACCEPT_PATCH, ACCEPT_POST};

/// Tower layer for client-side content negotiation.
///
/// Wraps an inner service to add content negotiation behavior:
/// - Serializes request bodies with the selected format
/// - Sets appropriate `Content-Type` headers
/// - Caches successful format selections for subsequent requests
#[derive(Debug, Clone)]
pub struct ClientNegotiateLayer {
    config: Arc<ClientConfig>,
}

impl ClientNegotiateLayer {
    /// Creates a new client negotiation layer with the given configuration.
    pub fn new(config: ClientConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }
}

impl<S> Layer<S> for ClientNegotiateLayer {
    type Service = ClientNegotiateService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ClientNegotiateService::new(inner, Arc::clone(&self.config))
    }
}

/// Tower service for client-side content negotiation.
///
/// This service wraps an HTTP client service and handles format negotiation:
/// - Selects the format to use (cached or highest priority)
/// - Caches successful formats for future requests
/// - Handles 415 responses by parsing Accept-Post/Accept-Patch headers
#[derive(Debug, Clone)]
pub struct ClientNegotiateService<S> {
    inner: S,
    config: Arc<ClientConfig>,
    cached_format: Arc<OnceLock<Arc<dyn ErasedFormat>>>,
}

impl<S> ClientNegotiateService<S> {
    /// Creates a new client negotiation service.
    pub fn new(inner: S, config: Arc<ClientConfig>) -> Self {
        Self {
            inner,
            config,
            cached_format: Arc::new(OnceLock::new()),
        }
    }

    /// Returns the currently cached format, if any.
    pub fn cached_format(&self) -> Option<Arc<dyn ErasedFormat>> {
        self.cached_format.get().cloned()
    }

    /// Gets the format to use for the request.
    ///
    /// Per DESIGN.md request flow:
    /// 1. If format already cached: use cached format
    /// 2. Otherwise, use highest-priority format from config
    fn select_request_format(&self) -> Arc<dyn ErasedFormat> {
        self.cached_format.get().cloned().unwrap_or_else(|| {
            self.config
                .formats
                .first()
                .cloned()
                .unwrap_or_else(|| self.config.fallback_format.clone())
        })
    }
}

/// Serializes a value using the given format into a request body.
///
/// # Errors
///
/// Returns an error if serialization fails.
pub fn serialize<T: serde::Serialize>(
    value: &T,
    format: &dyn ErasedFormat,
) -> Result<Full<Bytes>, erased_serde::Error> {
    let mut bytes = Vec::new();
    format.serialize(&mut bytes, &mut |serializer| {
        use erased_serde::Serialize;
        value.erased_serialize(serializer)
    })?;
    Ok(Full::new(bytes.into()))
}

/// Extension trait for requests to set the negotiated format.
pub trait ClientRequestExt<B> {
    /// Sets the content negotiation format for this request, bypassing the cache.
    #[must_use]
    fn with_format(self, format: Arc<dyn ErasedFormat>) -> Self;

    /// Gets the format override set on this request, if any.
    fn format_override(&self) -> Option<&Arc<dyn ErasedFormat>>;
}

/// Extension type stored in request extensions to override format selection.
#[derive(Debug, Clone)]
pub(crate) struct FormatOverride(pub Arc<dyn ErasedFormat>);

impl<B> ClientRequestExt<B> for Request<B> {
    fn with_format(mut self, format: Arc<dyn ErasedFormat>) -> Self {
        self.extensions_mut().insert(FormatOverride(format));
        self
    }

    fn format_override(&self) -> Option<&Arc<dyn ErasedFormat>> {
        self.extensions().get::<FormatOverride>().map(|o| &o.0)
    }
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for ClientNegotiateService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone,
    ReqBody: Body,
    ResBody: Body,
    ResBody::Error: std::fmt::Display,
{
    type Response = Response<ResBody>;
    type Error = S::Error;
    type Future = ClientNegotiateFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        // DESIGN.md step 1-2: Use cached format if available, else highest-priority
        let format = req
            .format_override()
            .cloned()
            .unwrap_or_else(|| self.select_request_format());

        let (mut parts, body) = req.into_parts();
        parts
            .headers
            .insert(header::CONTENT_TYPE, format.content_type_header());
        if let Some(accept) = self.config.accept_header_value.clone() {
            parts.headers.insert(header::ACCEPT, accept);
        }
        let new_req = Request::from_parts(parts, body);

        let config = Arc::clone(&self.config);
        let cached_format = Arc::clone(&self.cached_format);
        let inner_future = self.inner.call(new_req);

        ClientNegotiateFuture {
            inner: inner_future,
            format,
            config,
            cached_format,
        }
    }
}

pin_project! {
    /// Future returned by [`ClientNegotiateService`].
    pub struct ClientNegotiateFuture<F> {
        #[pin]
        inner: F,
        format: Arc<dyn ErasedFormat>,
        config: Arc<ClientConfig>,
        cached_format: Arc<OnceLock<Arc<dyn ErasedFormat>>>,
    }
}

impl<F, ResBody, E> Future for ClientNegotiateFuture<F>
where
    F: Future<Output = Result<Response<ResBody>, E>>,
{
    type Output = Result<Response<ResBody>, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        match this.inner.poll(cx) {
            Poll::Ready(Ok(response)) => {
                let status = response.status();

                // DESIGN.md step 3: On success (2xx), cache the format we used
                if status.is_success() {
                    // First writer wins - ignore if already cached
                    let _ = this.cached_format.set(Arc::clone(this.format));
                } else if status == StatusCode::UNSUPPORTED_MEDIA_TYPE {
                    // DESIGN.md step 4: On 415, parse Accept-Post/Accept-Patch
                    // and cache the server's preferred format
                    let accept_header = response
                        .headers()
                        .get(&ACCEPT_POST)
                        .or_else(|| response.headers().get(&ACCEPT_PATCH));

                    let new_format = accept_header
                        .and_then(|hv| hv.to_str().ok())
                        .and_then(|header_str| {
                            // Select highest-priority client format from server's list
                            parse_accept_erased(header_str, &this.config.formats).map(|m| m.format)
                        })
                        // If no header or no match, use fallback_format
                        .unwrap_or_else(|| Arc::clone(&this.config.fallback_format));

                    // First writer wins - ignore if already cached
                    let _ = this.cached_format.set(new_format);
                }

                Poll::Ready(Ok(response))
            }
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }
}
