//! Request extractor.

use std::ops::Deref;
use std::sync::Arc;

use serde::de::DeserializeOwned;

use super::{NegotiateResponse, NegotiatedFormat};
use crate::format::ErasedFormat;

/// Extractor and responder for content-negotiated requests.
///
/// Deserializes the request body using the negotiated format and provides
/// `respond()` to serialize responses with the same format.
pub struct Negotiate<T: DeserializeOwned> {
    value: T,
    format: Arc<dyn ErasedFormat>,
}

impl<T: DeserializeOwned> Negotiate<T> {
    /// Creates a new extractor with a deserialized value and format.
    pub fn new(value: T, format: Arc<dyn ErasedFormat>) -> Self {
        Self { value, format }
    }

    /// Creates a response using the negotiated format.
    pub fn respond<U: serde::Serialize>(&self, value: U) -> NegotiateResponse<U> {
        NegotiateResponse::new(value, Arc::clone(&self.format))
    }

    /// Returns the negotiated format.
    pub fn format(&self) -> &Arc<dyn ErasedFormat> {
        &self.format
    }

    /// Returns the inner value.
    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T: DeserializeOwned> Deref for Negotiate<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: DeserializeOwned + std::fmt::Debug> std::fmt::Debug for Negotiate<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Negotiate")
            .field("value", &self.value)
            .field("format", &self.format)
            .finish()
    }
}

/// Returns the negotiated format from request extensions, if present.
pub fn extract_negotiated_format(extensions: &http::Extensions) -> Option<&NegotiatedFormat> {
    extensions.get::<NegotiatedFormat>()
}
