//! Response wrapper for content-negotiated responses.

use std::sync::Arc;

use serde::Serialize;

use crate::ErasedFormat;

/// Response wrapper that holds a value and its serialization format.
///
/// The format is captured from the incoming request's Accept header negotiation.
/// Serialization happens in `IntoResponse` (with axum feature).
pub struct NegotiateResponse<T: Serialize> {
    value: T,
    format: Arc<dyn ErasedFormat>,
}

impl<T: Serialize> NegotiateResponse<T> {
    /// Creates a new negotiated response.
    pub fn new(value: T, format: Arc<dyn ErasedFormat>) -> Self {
        Self { value, format }
    }

    /// Returns the response format.
    pub fn format(&self) -> &Arc<dyn ErasedFormat> {
        &self.format
    }

    /// Consumes self and returns the inner value.
    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T: Serialize + std::fmt::Debug> std::fmt::Debug for NegotiateResponse<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NegotiateResponse")
            .field("value", &self.value)
            .field("format", &self.format)
            .finish()
    }
}
