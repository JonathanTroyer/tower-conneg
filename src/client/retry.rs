//! 415 retry helper.

use std::sync::Arc;

use http::{Request, Response, StatusCode};
use tower::Service;

use super::{parse_415_accept_header, select_initial_format};
use crate::core::ClientConfig;
use crate::format::ErasedFormat;

/// Retries requests with alternative formats on 415 responses.
#[derive(Debug, Clone)]
pub struct Retry415Helper {
    config: Arc<ClientConfig>,
    max_attempts: usize,
}

impl Retry415Helper {
    /// Creates a new helper with the given config and max attempts.
    pub fn new(config: ClientConfig, max_attempts: usize) -> Self {
        Self {
            config: Arc::new(config),
            max_attempts,
        }
    }

    /// Executes a request, retrying with different formats on 415.
    ///
    /// # Errors
    /// Returns `RetryError` if the service fails.
    pub async fn call<S, ReqBody, ResBody, F>(
        &self,
        mut service: S,
        mut request_fn: F,
    ) -> Result<Response<ResBody>, RetryError<S::Error>>
    where
        S: Service<Request<ReqBody>, Response = Response<ResBody>>,
        F: FnMut(Arc<dyn ErasedFormat>) -> Request<ReqBody>,
    {
        let mut format = select_initial_format(&self.config);

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

            format = parse_415_accept_header(&response, &self.config);
        }

        unreachable!("loop should return before exhausting")
    }
}

/// Error from [`Retry415Helper`].
#[derive(Debug)]
pub enum RetryError<E> {
    /// Service error.
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
