//! Extension traits for HTTP request builders and responses.

mod request;
mod response;

#[cfg(feature = "hyper-client")]
mod hyper;

pub use request::NegotiateRequestBuilderExt;
pub use response::NegotiateResponseExt;

#[cfg(feature = "hyper-client")]
pub use self::hyper::HyperClientExt;
