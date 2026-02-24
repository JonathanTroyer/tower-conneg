//! Tower middleware for HTTP content negotiation.
//!
//! Provides automatic format selection based on `Accept` and `Content-Type` headers
//! for both server and client use cases.

/// Accept-Post header (W3C LDP).
pub(crate) const ACCEPT_POST: http::HeaderName = http::HeaderName::from_static("accept-post");
/// Accept-Patch header (RFC 5789).
pub(crate) const ACCEPT_PATCH: http::HeaderName = http::HeaderName::from_static("accept-patch");

mod client;
mod core;
mod ext;
mod format;
mod server;

// Core re-exports
pub use core::{
    AcceptMatch, ClientConfig, Negotiate, NegotiateResponse, NegotiatedFormat, NegotiationError,
    ServerConfig, extract_negotiated_format, parse_accept, parse_content_type,
    parse_content_type_erased,
};

// Client re-exports
pub use client::{
    ClientNegotiateFuture, ClientNegotiateLayer, ClientNegotiateService, ClientRequestExt,
    Retry415Helper, RetryError, serialize,
};

// Server re-exports
pub use server::{NegotiateLayer, NegotiateService};

// Extension re-exports
#[cfg(feature = "hyper-client")]
pub use ext::HyperClientExt;
pub use ext::NegotiateRequestBuilderExt;
pub use ext::NegotiateResponseExt;

// Format re-exports
#[cfg(feature = "bson")]
pub use format::BsonFormat;
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
#[cfg(feature = "postcard")]
pub use format::PostcardFormat;
#[cfg(feature = "toml")]
pub use format::TomlFormat;
#[cfg(feature = "xml")]
pub use format::XmlFormat;
pub use format::{
    Borrowable, Consumable, ErasedFormat, Format, MatchSpecificity, OwnedDeserializer,
    OwnedSerializer, match_specificity,
};
