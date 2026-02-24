//! Error types.

use http::StatusCode;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("deserialization produced no value")]
pub(crate) struct DeserializationProducedNoValue;

/// Content negotiation error.
#[derive(Debug, Clone, thiserror::Error)]
pub enum NegotiationError {
    /// 406 Not Acceptable.
    #[error("no acceptable response format found (requested: {requested}; supported: {supported})")]
    NotAcceptable {
        /// The Accept header value, or "none provided" if absent.
        requested: Arc<str>,
        /// Comma-separated list of supported media types.
        supported: Arc<str>,
    },
    /// 415 Unsupported Media Type.
    #[error("request content-type not supported (provided: {provided}; supported: {supported})")]
    UnsupportedMediaType {
        /// The Content-Type header value, or "none provided" if absent.
        provided: Arc<str>,
        /// Comma-separated list of supported media types.
        supported: Arc<str>,
    },
    /// Serialization failed.
    #[error("serialization failed: {0}")]
    Serialization(Arc<dyn std::error::Error + Send + Sync>),
    /// Deserialization failed.
    #[error("deserialization failed: {0}")]
    Deserialization(Arc<dyn std::error::Error + Send + Sync>),
    /// Body collection failed.
    #[error("failed to collect response body: {0}")]
    BodyCollection(Arc<dyn std::error::Error + Send + Sync>),
}

impl NegotiationError {
    /// Returns the corresponding HTTP status code.
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::NotAcceptable { .. } => StatusCode::NOT_ACCEPTABLE,
            Self::UnsupportedMediaType { .. } => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            Self::Serialization(_) | Self::Deserialization(_) | Self::BodyCollection(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    /// Creates a `NotAcceptable` error.
    pub fn not_acceptable(requested: Option<&str>, supported: &[String]) -> Self {
        Self::NotAcceptable {
            requested: requested.unwrap_or("none provided").into(),
            supported: supported.join(", ").into(),
        }
    }

    /// Creates an `UnsupportedMediaType` error.
    pub fn unsupported_media_type(provided: Option<&str>, supported: &[String]) -> Self {
        Self::UnsupportedMediaType {
            provided: provided.unwrap_or("none provided").into(),
            supported: supported.join(", ").into(),
        }
    }

    /// Creates a `Serialization` error.
    pub fn serialization(err: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::Serialization(Arc::new(err))
    }

    /// Creates a `Deserialization` error.
    pub fn deserialization(err: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::Deserialization(Arc::new(err))
    }

    /// Creates a `BodyCollection` error.
    pub fn body_collection(err: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::BodyCollection(Arc::new(err))
    }

    pub(crate) fn deserialization_produced_no_value() -> Self {
        Self::Deserialization(Arc::new(DeserializationProducedNoValue))
    }
}
