//! Content negotiation middleware for HTTP.

/// Accept-Post header (W3C LDP).
pub(crate) const ACCEPT_POST: http::HeaderName = http::HeaderName::from_static("accept-post");
/// Accept-Patch header (RFC 5789).
pub(crate) const ACCEPT_PATCH: http::HeaderName = http::HeaderName::from_static("accept-patch");

mod accept;
mod client;
mod config;
mod content_type;
mod error;
mod ext;
mod extractor;
mod format;
mod negotiated;
mod response;
mod retry;
mod server;

pub use accept::{AcceptMatch, parse_accept};
pub use client::{
    ClientNegotiateFuture, ClientNegotiateLayer, ClientNegotiateService, ClientRequestExt,
    serialize,
};
pub use config::{ClientConfig, ServerConfig};
pub use content_type::{parse_content_type, parse_content_type_erased};
pub use error::NegotiationError;
#[cfg(feature = "hyper-client")]
pub use ext::HyperClientExt;
pub use ext::NegotiateRequestBuilderExt;
pub use ext::NegotiateResponseExt;
pub use extractor::{Negotiate, extract_negotiated_format};
#[cfg(feature = "cbor")]
pub use format::CborFormat;
#[cfg(feature = "form")]
pub use format::FormFormat;
#[cfg(feature = "plain")]
pub use format::HtmlFormat;
#[cfg(feature = "json")]
pub use format::JsonFormat;
#[cfg(feature = "msgpack")]
pub use format::MsgPackFormat;
#[cfg(feature = "plain")]
pub use format::PlainTextFormat;
#[cfg(feature = "xml")]
pub use format::XmlFormat;
pub use format::{
    Borrowable, Consumable, ErasedFormat, Format, MatchSpecificity, OwnedDeserializer,
    OwnedSerializer, match_specificity,
};
pub use negotiated::NegotiatedFormat;
pub use response::NegotiateResponse;
pub use retry::{Retry415Helper, RetryError};
pub use server::{NegotiateLayer, NegotiateService};
