//! Object-safe format trait.

use erased_serde::{Deserializer, Serializer};
use http::HeaderValue;
use mediatype::MediaType;
use sealed::sealed;
use std::fmt::{self, Debug, Formatter};

use super::{Format, MatchSpecificity, OwnedDeserializer, OwnedSerializer as _};

/// Object-safe version of [`Format`] for dynamic dispatch.
///
/// Automatically implemented for all `Format` types.
#[sealed]
pub trait ErasedFormat: Send + Sync {
    /// Primary media type for this format.
    fn primary_media_type(&self) -> MediaType<'static>;

    /// Content-Type header value.
    fn content_type_header(&self) -> HeaderValue;

    /// Match specificity against a requested media type.
    fn match_specificity(&self, requested: &MediaType<'_>) -> Option<MatchSpecificity>;

    /// Serializes via a callback receiving a type-erased serializer.
    ///
    /// # Errors
    /// Returns an error if serialization fails.
    fn serialize(
        &self,
        bytes: &mut Vec<u8>,
        body: &mut dyn FnMut(&mut dyn Serializer) -> erased_serde::Result<()>,
    ) -> erased_serde::Result<()>;

    /// Deserializes via a callback receiving a type-erased deserializer.
    ///
    /// # Errors
    /// Returns an error if deserialization fails.
    fn deserialize(
        &self,
        bytes: &[u8],
        body: &mut dyn FnMut(&mut dyn Deserializer<'_>) -> erased_serde::Result<()>,
    ) -> erased_serde::Result<()>;
}

#[sealed]
impl<F> ErasedFormat for F
where
    F: Format + Send + Sync,
{
    fn primary_media_type(&self) -> MediaType<'static> {
        self.media_types()
            .first()
            .cloned()
            .unwrap_or(mediatype::media_type!(APPLICATION / OCTET_STREAM))
    }

    fn content_type_header(&self) -> HeaderValue {
        <F as Format>::content_type_header(self)
    }

    fn match_specificity(&self, requested: &MediaType<'_>) -> Option<MatchSpecificity> {
        super::match_specificity(self, requested)
    }

    fn serialize(
        &self,
        bytes: &mut Vec<u8>,
        body: &mut dyn FnMut(&mut dyn Serializer) -> erased_serde::Result<()>,
    ) -> erased_serde::Result<()> {
        let serializer = <F as Format>::serializer(self, bytes)?;
        serializer.with_erased(body)
    }

    fn deserialize(
        &self,
        bytes: &[u8],
        body: &mut dyn FnMut(&mut dyn Deserializer<'_>) -> erased_serde::Result<()>,
    ) -> erased_serde::Result<()> {
        let deserializer = <F as Format>::deserializer(self, bytes)?;
        let mut erased = <dyn Deserializer<'_>>::erase(deserializer.into_deserializer());
        body(&mut erased)
    }
}

impl Debug for dyn ErasedFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "ErasedFormat({})", self.primary_media_type())
    }
}
