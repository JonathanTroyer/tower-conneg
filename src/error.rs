//! Error types for content negotiation failures.

use http::StatusCode;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

/// Errors that can occur during content negotiation.
#[derive(Debug, Clone)]
pub enum NegotiationError {
    /// No acceptable response format found (406 Not Acceptable).
    ///
    /// Returned when the `Accept` header doesn't match any supported format.
    /// Contains the requested `Accept` header value (if present) and the list
    /// of supported media types for debugging.
    NotAcceptable {
        /// The `Accept` header value that was sent, or `None` if no header was present.
        requested: Option<String>,
        /// List of supported media types.
        supported: Arc<[String]>,
    },
    /// Request Content-Type not supported (415 Unsupported Media Type).
    ///
    /// Returned when the `Content-Type` header doesn't match any supported format.
    /// Contains the provided `Content-Type` header value (if present) and the list
    /// of supported media types for debugging.
    UnsupportedMediaType {
        /// The `Content-Type` header value that was sent, or `None` if missing.
        provided: Option<String>,
        /// List of supported media types.
        supported: Arc<[String]>,
    },
    /// Serialization failed.
    Serialization {
        /// The serialization error message.
        source: String,
    },
    /// Deserialization failed.
    Deserialization {
        /// The deserialization error message.
        source: String,
    },
    /// Failed to collect response body.
    BodyCollection {
        /// The body collection error message.
        source: String,
    },
}

impl NegotiationError {
    /// Returns the HTTP status code for this error.
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::NotAcceptable { .. } => StatusCode::NOT_ACCEPTABLE,
            Self::UnsupportedMediaType { .. } => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            Self::Serialization { .. }
            | Self::Deserialization { .. }
            | Self::BodyCollection { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl fmt::Display for NegotiationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAcceptable {
                requested,
                supported,
            } => {
                let requested_str = requested.as_deref().unwrap_or("none provided");
                let supported_str = supported.join(", ");
                write!(
                    f,
                    "no acceptable response format found (requested: {requested_str}; supported: {supported_str})"
                )
            }
            Self::UnsupportedMediaType {
                provided,
                supported,
            } => {
                let provided_str = provided.as_deref().unwrap_or("none provided");
                let supported_str = supported.join(", ");
                write!(
                    f,
                    "request content-type not supported (provided: {provided_str}; supported: {supported_str})"
                )
            }
            Self::Serialization { source } => {
                write!(f, "serialization failed: {source}")
            }
            Self::Deserialization { source } => {
                write!(f, "deserialization failed: {source}")
            }
            Self::BodyCollection { source } => {
                write!(f, "failed to collect response body: {source}")
            }
        }
    }
}

impl Error for NegotiationError {}
