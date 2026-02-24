//! Negotiated format storage.

use std::sync::Arc;

use crate::format::ErasedFormat;

/// Negotiated formats stored in request extensions.
///
/// Contains the response format (from Accept) and optionally the request
/// format (from Content-Type). Typically accessed via the `Negotiate<T>` extractor.
#[derive(Debug, Clone)]
pub struct NegotiatedFormat {
    response_format: Arc<dyn ErasedFormat>,
    request_format: Option<Arc<dyn ErasedFormat>>,
}

impl NegotiatedFormat {
    /// Creates with response format only (for GET, DELETE, etc.).
    pub fn response_only(response_format: Arc<dyn ErasedFormat>) -> Self {
        Self {
            response_format,
            request_format: None,
        }
    }

    /// Creates with both response and request formats (for POST, PUT, PATCH).
    pub fn with_request_format(
        response_format: Arc<dyn ErasedFormat>,
        request_format: Arc<dyn ErasedFormat>,
    ) -> Self {
        Self {
            response_format,
            request_format: Some(request_format),
        }
    }

    /// Format for serializing responses.
    pub fn response_format(&self) -> &Arc<dyn ErasedFormat> {
        &self.response_format
    }

    /// Format for deserializing request bodies, if present.
    pub fn request_format(&self) -> Option<&Arc<dyn ErasedFormat>> {
        self.request_format.as_ref()
    }
}
