//! Format information stored in request extensions by the middleware.

use std::sync::Arc;

use crate::ErasedFormat;

/// Format information stored in request extensions by the middleware.
///
/// The middleware parses `Accept` and `Content-Type` headers, selects the
/// appropriate formats, and stores this type in the request extensions.
/// Extractors (like `Negotiate<T>`) read from extensions to access format info.
///
/// Users typically interact with this through the `Negotiate<T>` extractor.
#[derive(Debug, Clone)]
pub struct NegotiatedFormat {
    /// Format for serializing responses (selected from Accept header).
    response_format: Arc<dyn ErasedFormat>,
    /// Format for deserializing request body (from Content-Type header).
    /// None if request has no body or Content-Type header.
    request_format: Option<Arc<dyn ErasedFormat>>,
}

impl NegotiatedFormat {
    /// Creates a new `NegotiatedFormat` with only a response format.
    ///
    /// Use this for requests without a body (GET, DELETE, etc.) where no
    /// Content-Type parsing is needed.
    pub fn response_only(response_format: Arc<dyn ErasedFormat>) -> Self {
        Self {
            response_format,
            request_format: None,
        }
    }

    /// Creates a new `NegotiatedFormat` with both response and request formats.
    ///
    /// Use this for requests with a body (POST, PUT, PATCH) where both Accept
    /// and Content-Type headers are processed.
    pub fn with_request_format(
        response_format: Arc<dyn ErasedFormat>,
        request_format: Arc<dyn ErasedFormat>,
    ) -> Self {
        Self {
            response_format,
            request_format: Some(request_format),
        }
    }

    /// Returns the format for serializing responses.
    pub fn response_format(&self) -> &Arc<dyn ErasedFormat> {
        &self.response_format
    }

    /// Returns the format for deserializing request bodies, if present.
    pub fn request_format(&self) -> Option<&Arc<dyn ErasedFormat>> {
        self.request_format.as_ref()
    }
}
