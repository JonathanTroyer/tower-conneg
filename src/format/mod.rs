//! Serialization format traits and implementations.

#[cfg(any(feature = "form", feature = "plain"))]
#[macro_use]
mod macros;

#[cfg(feature = "bson")]
mod bson;
#[cfg(feature = "cbor")]
mod cbor;
mod erased;
#[cfg(feature = "form")]
mod form;
#[cfg(feature = "plain")]
mod html;
#[cfg(feature = "json")]
mod json;
#[cfg(feature = "msgpack")]
mod msgpack;
mod owned_deserializer;
mod owned_serializer;
#[cfg(feature = "plain")]
mod plain_text;
#[cfg(feature = "postcard")]
mod postcard;
#[cfg(feature = "toml")]
mod toml;
#[cfg(feature = "xml")]
mod xml;

#[cfg(feature = "bson")]
pub use bson::BsonFormat;
#[cfg(feature = "cbor")]
pub use cbor::CborFormat;
pub use erased::ErasedFormat;
#[cfg(feature = "form")]
pub use form::FormFormat;
#[cfg(feature = "plain")]
pub use html::HtmlFormat;
#[cfg(feature = "json")]
pub use json::JsonFormat;
#[cfg(feature = "msgpack")]
pub use msgpack::{MsgPackFormat, MsgPackNamedFormat};
pub use owned_deserializer::{Borrowable, Consumable, OwnedDeserializer};
pub use owned_serializer::OwnedSerializer;
#[cfg(feature = "plain")]
pub use plain_text::PlainTextFormat;
#[cfg(feature = "postcard")]
pub use postcard::PostcardFormat;
#[cfg(feature = "toml")]
pub use toml::TomlFormat;
#[cfg(feature = "xml")]
pub use xml::XmlFormat;

use http::HeaderValue;
use mediatype::{MediaType, names};

/// Media type match specificity level.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum MatchSpecificity {
    /// `*/*` wildcard.
    Wildcard,
    /// Type wildcard (e.g., `application/*`).
    TypeOnly,
    /// Exact type and subtype.
    Exact,
}

/// Serialization format for content negotiation.
pub trait Format {
    /// Media types this format handles.
    fn media_types(&self) -> &'static [MediaType<'static>];

    /// Content-Type header value (must be static to avoid allocation).
    fn content_type_header(&self) -> HeaderValue;

    /// Creates a serializer writing to the given buffer.
    ///
    /// # Errors
    /// Returns an error if creation fails.
    fn serializer<'a>(
        &'a self,
        bytes: &'a mut Vec<u8>,
    ) -> erased_serde::Result<impl OwnedSerializer + 'a>;

    /// Creates a deserializer reading from the given bytes.
    ///
    /// # Errors
    /// Returns an error if creation fails.
    fn deserializer<'a>(
        &'a self,
        bytes: &'a [u8],
    ) -> erased_serde::Result<impl OwnedDeserializer<'a> + 'a>;
}

/// Returns the match specificity between a format and a requested media type.
pub fn match_specificity<F: Format + ?Sized>(
    format: &F,
    requested: &MediaType<'_>,
) -> Option<MatchSpecificity> {
    if requested.ty == names::_STAR && requested.subty == names::_STAR {
        return Some(MatchSpecificity::Wildcard);
    }

    let mut type_match = false;
    for mt in format.media_types() {
        if mt.ty == requested.ty {
            if mt.subty == requested.subty {
                return Some(MatchSpecificity::Exact);
            }
            type_match = true;
        }
    }
    if requested.subty == names::_STAR && type_match {
        return Some(MatchSpecificity::TypeOnly);
    }

    None
}
