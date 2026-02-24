//! Request extractor for content-negotiated requests.

use std::ops::Deref;
use std::sync::Arc;

use serde::de::DeserializeOwned;

use crate::{ErasedFormat, NegotiateResponse, NegotiatedFormat};

/// Combined extractor and responder for content-negotiated requests.
///
/// `Negotiate<T>` captures the negotiated format from request extensions and
/// deserializes the request body (when `T` is not `()`). It provides a
/// `.respond()` method to create a response using the same format negotiation.
///
/// # Usage
///
/// For requests with a body (POST, PUT, PATCH):
/// ```ignore
/// async fn create_user(req: Negotiate<CreateUserRequest>) -> NegotiateResponse<User> {
///     let user = db.create(&req).await;  // Deref to access body
///     req.respond(user)                   // Creates response with captured format
/// }
/// ```
///
/// For requests without a body (GET, DELETE):
/// ```ignore
/// async fn get_user(neg: Negotiate<()>, Path(id): Path<u64>) -> NegotiateResponse<User> {
///     let user = db.get(id).await;
///     neg.respond(user)
/// }
/// ```
pub struct Negotiate<T: DeserializeOwned> {
    value: T,
    format: Arc<dyn ErasedFormat>,
}

impl<T: DeserializeOwned> Negotiate<T> {
    /// Creates a new `Negotiate` with a deserialized value and format.
    pub fn new(value: T, format: Arc<dyn ErasedFormat>) -> Self {
        Self { value, format }
    }

    /// Creates a response using the captured format negotiation.
    ///
    /// The returned `NegotiateResponse<U>` will serialize `value` using the
    /// response format that was negotiated from the `Accept` header.
    pub fn respond<U: serde::Serialize>(&self, value: U) -> NegotiateResponse<U> {
        NegotiateResponse::new(value, Arc::clone(&self.format))
    }

    /// Returns the negotiated response format.
    pub fn format(&self) -> &Arc<dyn ErasedFormat> {
        &self.format
    }

    /// Consumes self and returns the inner value.
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

/// Extracts the negotiated format from request extensions.
///
/// Returns the `NegotiatedFormat` if present, or `None` if the negotiation
/// middleware hasn't processed this request.
pub fn extract_negotiated_format(extensions: &http::Extensions) -> Option<&NegotiatedFormat> {
    extensions.get::<NegotiatedFormat>()
}
