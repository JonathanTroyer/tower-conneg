//! Hyper client integration.
//!
//! Provides ergonomic wrappers for using content negotiation with hyper clients.
//! Since hyper-util's `Client` already implements Tower's `Service` trait, this
//! module provides a convenience extension trait that applies [`ClientNegotiateLayer`].
//!
//! # Example
//!
//! ```ignore
//! use hyper_util::client::legacy::Client;
//! use hyper_util::rt::TokioExecutor;
//! use tower_conneg::{ClientConfig, HyperClientExt, JsonFormat};
//! use std::sync::Arc;
//!
//! let config = ClientConfig::builder()
//!     .formats([Arc::new(JsonFormat) as _])
//!     .fallback_format(Arc::new(JsonFormat))
//!     .build();
//!
//! let client = Client::builder(TokioExecutor::new())
//!     .build_http()
//!     .with_content_negotiation(config);
//! ```

use tower::Layer;

use crate::client::{ClientNegotiateLayer, ClientNegotiateService};
use crate::config::ClientConfig;

/// Extension trait for wrapping Tower services with content negotiation.
///
/// This trait is implemented for all types, making it easy to wrap any
/// Tower-compatible HTTP client with content negotiation middleware.
pub trait HyperClientExt: Sized {
    /// Wraps the service with content negotiation middleware.
    ///
    /// This applies [`ClientNegotiateLayer`] to the service, which:
    /// - Sets `Content-Type` headers on requests
    /// - Sets `Accept` headers on requests
    /// - Caches successful formats for subsequent requests
    /// - Parses `Accept-Post`/`Accept-Patch` headers on 415 responses
    fn with_content_negotiation(self, config: ClientConfig) -> ClientNegotiateService<Self>;
}

impl<S> HyperClientExt for S {
    fn with_content_negotiation(self, config: ClientConfig) -> ClientNegotiateService<Self> {
        ClientNegotiateLayer::new(config).layer(self)
    }
}
