//! Hyper client integration.

use tower::Layer;

use crate::client::{ClientNegotiateLayer, ClientNegotiateService};
use crate::core::ClientConfig;

/// Extension trait for wrapping Tower services with content negotiation.
pub trait HyperClientExt: Sized {
    /// Wraps the service with content negotiation middleware.
    fn with_content_negotiation(self, config: ClientConfig) -> ClientNegotiateService<Self>;
}

impl<S> HyperClientExt for S {
    fn with_content_negotiation(self, config: ClientConfig) -> ClientNegotiateService<Self> {
        ClientNegotiateLayer::new(config).layer(self)
    }
}
