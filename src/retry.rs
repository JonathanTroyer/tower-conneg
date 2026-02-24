//! Retry helper for 415 Unsupported Media Type responses.

use std::sync::Arc;

use http::{Request, Response, StatusCode};
use tower::Service;

use crate::accept::parse_accept_erased;
use crate::config::ClientConfig;
use crate::format::ErasedFormat;
use crate::{ACCEPT_PATCH, ACCEPT_POST};

/// Helper for retrying requests with different formats on 415 responses.
///
/// Unlike [`ClientNegotiateService`](crate::ClientNegotiateService) which only caches
/// the format for future requests, this helper actively retries the current request
/// with alternative formats when a 415 is received.
#[derive(Debug, Clone)]
pub struct Retry415Helper {
    config: Arc<ClientConfig>,
    max_attempts: usize,
}

impl Retry415Helper {
    /// Creates a new retry helper.
    ///
    /// # Arguments
    ///
    /// * `config` - Client configuration with supported formats
    /// * `max_attempts` - Maximum number of attempts (including initial request)
    pub fn new(config: ClientConfig, max_attempts: usize) -> Self {
        Self {
            config: Arc::new(config),
            max_attempts,
        }
    }

    /// Executes a request, retrying with different formats on 415.
    ///
    /// The `request_fn` closure is called with the format to use and should
    /// return a new request. This allows serializing the body with each format.
    ///
    /// On success (any non-415 status), returns the response immediately.
    /// On 415, parses `Accept-Post` or `Accept-Patch` headers to find an
    /// alternative format and retries.
    ///
    /// # Errors
    ///
    /// Returns [`RetryError::Service`] if the underlying service returns an error.
    pub async fn call<S, ReqBody, ResBody, F>(
        &self,
        mut service: S,
        mut request_fn: F,
    ) -> Result<Response<ResBody>, RetryError<S::Error>>
    where
        S: Service<Request<ReqBody>, Response = Response<ResBody>>,
        F: FnMut(Arc<dyn ErasedFormat>) -> Request<ReqBody>,
    {
        let mut format = self
            .config
            .formats
            .first()
            .cloned()
            .unwrap_or_else(|| self.config.fallback_format.clone());

        for attempt in 0..self.max_attempts {
            let request = request_fn(Arc::clone(&format));

            std::future::poll_fn(|cx| service.poll_ready(cx))
                .await
                .map_err(RetryError::Service)?;

            let response = service.call(request).await.map_err(RetryError::Service)?;

            if response.status() != StatusCode::UNSUPPORTED_MEDIA_TYPE {
                return Ok(response);
            }

            // Last attempt - return the 415 response
            if attempt + 1 >= self.max_attempts {
                return Ok(response);
            }

            // Parse Accept-Post/Accept-Patch to find alternative format
            let accept_header = response
                .headers()
                .get(&ACCEPT_POST)
                .or_else(|| response.headers().get(&ACCEPT_PATCH));

            format = accept_header
                .and_then(|hv| hv.to_str().ok())
                .and_then(|header_str| {
                    parse_accept_erased(header_str, &self.config.formats).map(|m| m.format)
                })
                .unwrap_or_else(|| self.config.fallback_format.clone());
        }

        unreachable!("loop should return before exhausting")
    }
}

/// Error from retry helper.
#[derive(Debug)]
pub enum RetryError<E> {
    /// The underlying service returned an error.
    Service(E),
}

impl<E: std::fmt::Display> std::fmt::Display for RetryError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Service(e) => write!(f, "service error: {e}"),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for RetryError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Service(e) => Some(e),
        }
    }
}
