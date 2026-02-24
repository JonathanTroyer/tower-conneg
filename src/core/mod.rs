//! Core types for content negotiation.

mod accept;
mod config;
mod content_type;
mod error;
mod extractor;
mod negotiated;
mod response;

pub(crate) use accept::parse_accept_erased;
pub use accept::{AcceptMatch, parse_accept};
pub use config::{ClientConfig, ServerConfig};
pub use content_type::{parse_content_type, parse_content_type_erased};
pub use error::NegotiationError;
pub use extractor::{Negotiate, extract_negotiated_format};
pub use negotiated::NegotiatedFormat;
pub use response::NegotiateResponse;
