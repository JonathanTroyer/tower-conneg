//! Server-side content negotiation middleware.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use http::{HeaderValue, Method, Request, Response, StatusCode, header};
use pin_project_lite::pin_project;
use tower::{Layer, Service};

use crate::core::{
    NegotiatedFormat, NegotiationError, ServerConfig, parse_accept_erased,
    parse_content_type_erased,
};
use crate::format::MatchSpecificity;
use crate::{ACCEPT_PATCH, ACCEPT_POST};

/// Tower layer for server-side content negotiation.
///
/// Stores negotiated formats in request extensions for downstream extractors.
#[derive(Debug, Clone)]
pub struct NegotiateLayer {
    config: Arc<ServerConfig>,
}

impl NegotiateLayer {
    /// Creates a new layer with the given configuration.
    pub fn new(config: ServerConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }
}

impl<S> Layer<S> for NegotiateLayer {
    type Service = NegotiateService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        NegotiateService {
            inner,
            config: Arc::clone(&self.config),
        }
    }
}

/// Tower service for server-side content negotiation.
///
/// Parses headers and stores [`NegotiatedFormat`] in request extensions.
#[derive(Debug, Clone)]
pub struct NegotiateService<S> {
    inner: S,
    config: Arc<ServerConfig>,
}

impl<S> NegotiateService<S> {
    /// Wraps a service with content negotiation.
    pub fn new(inner: S, config: Arc<ServerConfig>) -> Self {
        Self { inner, config }
    }
}

enum NegotiateResult {
    Success(NegotiatedFormat),
    NotAcceptable,
    UnsupportedMediaType,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for NegotiateService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
    ResBody: Default,
{
    type Response = Response<ResBody>;
    type Error = S::Error;
    type Future = NegotiateFuture<S::Future, ResBody>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut request: Request<ReqBody>) -> Self::Future {
        match self.negotiate(&request) {
            NegotiateResult::Success(negotiated) => {
                request.extensions_mut().insert(negotiated);
                NegotiateFuture::Inner {
                    inner: self.inner.call(request),
                }
            }
            NegotiateResult::NotAcceptable => NegotiateFuture::NotAcceptable {
                _marker: std::marker::PhantomData,
            },
            NegotiateResult::UnsupportedMediaType => {
                let info = UnsupportedMediaTypeInfo {
                    method: request.method().clone(),
                    accept_header_value: self.config.accept_header_value.clone(),
                };
                NegotiateFuture::UnsupportedMediaType { info: Some(info) }
            }
        }
    }
}

impl<S> NegotiateService<S> {
    fn negotiate<ReqBody>(&self, request: &Request<ReqBody>) -> NegotiateResult {
        let Ok(response_format) = self.negotiate_response_format(request) else {
            return NegotiateResult::NotAcceptable;
        };

        let Ok(request_format) = self.negotiate_request_format(request) else {
            return NegotiateResult::UnsupportedMediaType;
        };

        let negotiated = match request_format {
            Some(f) => NegotiatedFormat::with_request_format(response_format, f),
            None => NegotiatedFormat::response_only(response_format),
        };

        NegotiateResult::Success(negotiated)
    }

    fn negotiate_response_format<ReqBody>(
        &self,
        request: &Request<ReqBody>,
    ) -> Result<Arc<dyn crate::format::ErasedFormat>, NegotiationError> {
        let accept_header = request
            .headers()
            .get(header::ACCEPT)
            .and_then(|v| v.to_str().ok());

        match accept_header {
            Some(accept) => {
                let matched = parse_accept_erased(accept, &self.config.formats);
                match matched {
                    Some(m) if m.specificity == MatchSpecificity::Wildcard => {
                        // Wildcard match - use fallback format as default
                        Ok(self.config.fallback_format.clone())
                    }
                    Some(m) => Ok(m.format),
                    None => {
                        // No match found
                        if self.config.strict {
                            Err(NegotiationError::not_acceptable(
                                Some(accept),
                                &self.config.supported_media_types,
                            ))
                        } else {
                            Ok(self.config.fallback_format.clone())
                        }
                    }
                }
            }
            None => {
                // No Accept header - use fallback format as default
                Ok(self.config.fallback_format.clone())
            }
        }
    }

    fn negotiate_request_format<ReqBody>(
        &self,
        request: &Request<ReqBody>,
    ) -> Result<Option<Arc<dyn crate::format::ErasedFormat>>, NegotiationError> {
        let content_type = request
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok());

        match content_type {
            Some(ct) => {
                let matched = parse_content_type_erased(ct, &self.config.formats);
                match matched {
                    Some(f) => Ok(Some(f)),
                    None => Err(NegotiationError::unsupported_media_type(
                        Some(ct),
                        &self.config.supported_media_types,
                    )),
                }
            }
            None => Ok(None), // No Content-Type header
        }
    }
}

struct UnsupportedMediaTypeInfo {
    method: Method,
    accept_header_value: Option<HeaderValue>,
}

pin_project! {
    /// Future returned by [`NegotiateService`].
    #[project = NegotiateFutureProj]
    pub enum NegotiateFuture<F, ResBody> {
        Inner { #[pin] inner: F },
        NotAcceptable { _marker: std::marker::PhantomData<ResBody> },
        UnsupportedMediaType { info: Option<UnsupportedMediaTypeInfo> },
    }
}

impl<F, ResBody, E> Future for NegotiateFuture<F, ResBody>
where
    F: Future<Output = Result<Response<ResBody>, E>>,
    ResBody: Default,
{
    type Output = Result<Response<ResBody>, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.project() {
            NegotiateFutureProj::Inner { inner } => inner.poll(cx),
            NegotiateFutureProj::NotAcceptable { .. } => {
                let response = Response::builder()
                    .status(StatusCode::NOT_ACCEPTABLE)
                    .body(ResBody::default())
                    .unwrap_or_else(|_| {
                        let mut r = Response::new(ResBody::default());
                        *r.status_mut() = StatusCode::NOT_ACCEPTABLE;
                        r
                    });
                Poll::Ready(Ok(response))
            }
            NegotiateFutureProj::UnsupportedMediaType { info } => {
                let info = info.take();
                let mut builder = Response::builder().status(StatusCode::UNSUPPORTED_MEDIA_TYPE);

                if let Some(info) = info
                    && let Some(accept_value) = info.accept_header_value
                {
                    let header_name = match info.method {
                        Method::POST => Some(ACCEPT_POST),
                        Method::PATCH => Some(ACCEPT_PATCH),
                        _ => None,
                    };
                    if let Some(name) = header_name {
                        builder = builder.header(name, accept_value);
                    }
                }

                let response = builder.body(ResBody::default()).unwrap_or_else(|_| {
                    let mut r = Response::new(ResBody::default());
                    *r.status_mut() = StatusCode::UNSUPPORTED_MEDIA_TYPE;
                    r
                });
                Poll::Ready(Ok(response))
            }
        }
    }
}
