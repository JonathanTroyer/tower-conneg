//! Format trait and matching logic.

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
#[cfg(feature = "xml")]
mod xml;

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
pub use msgpack::MsgPackFormat;
pub use owned_deserializer::{Borrowable, Consumable, OwnedDeserializer};
pub use owned_serializer::OwnedSerializer;
#[cfg(feature = "plain")]
pub use plain_text::PlainTextFormat;
#[cfg(feature = "xml")]
pub use xml::XmlFormat;

use http::HeaderValue;
use mediatype::{MediaType, names};

/// The specificity level of a media type match.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum MatchSpecificity {
    /// Matched via `*/*` wildcard.
    Wildcard,
    /// Matched via type wildcard (e.g., `application/*`).
    TypeOnly,
    /// Exact type and subtype match.
    Exact,
}

/// A serialization format for content negotiation.
pub trait Format {
    /// Returns the list of media types supported by this format.
    fn media_types(&self) -> &'static [MediaType<'static>];

    /// Returns the `Content-Type` header value for this format.
    ///
    /// Implementations must return a static `HeaderValue` (via `HeaderValue::from_static`)
    /// to avoid allocation on every request.
    fn content_type_header(&self) -> HeaderValue;

    /// Creates a serializer that writes to the given byte buffer.
    ///
    /// # Errors
    ///
    /// Returns an error if the serializer cannot be created.
    fn serializer<'a>(
        &'a self,
        bytes: &'a mut Vec<u8>,
    ) -> erased_serde::Result<impl OwnedSerializer + 'a>;

    /// Creates a deserializer that reads from the given bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if the deserializer cannot be created.
    fn deserializer<'a>(
        &'a self,
        bytes: &'a [u8],
    ) -> erased_serde::Result<impl OwnedDeserializer<'a> + 'a>;
}

/// Determines how specifically a format matches the requested media type.
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
