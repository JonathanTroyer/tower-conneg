//! Response wrapper.

use std::sync::Arc;

use serde::Serialize;

use crate::format::ErasedFormat;

/// Response wrapper holding a value and its serialization format.
pub struct NegotiateResponse<T: Serialize> {
    value: T,
    format: Arc<dyn ErasedFormat>,
}

impl<T: Serialize> NegotiateResponse<T> {
    /// Creates a new response with the given value and format.
    pub fn new(value: T, format: Arc<dyn ErasedFormat>) -> Self {
        Self { value, format }
    }

    /// Returns the serialization format.
    pub fn format(&self) -> &Arc<dyn ErasedFormat> {
        &self.format
    }

    /// Returns the inner value.
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
