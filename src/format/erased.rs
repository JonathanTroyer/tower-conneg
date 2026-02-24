//! Object-safe format trait for dynamic dispatch.

use erased_serde::{Deserializer, Serializer};
use http::HeaderValue;
use mediatype::MediaType;
use sealed::sealed;
use std::fmt::{self, Debug, Formatter};

use super::{Format, MatchSpecificity, OwnedDeserializer, OwnedSerializer};

/// Object-safe version of the [`Format`] trait.
///
/// This trait enables dynamic dispatch for format types, allowing them to be
/// stored in trait objects like `Arc<dyn ErasedFormat>`. The middleware uses
/// this to serialize responses without knowing the concrete format type.
///
/// You don't implement this trait directly - there's a blanket implementation
/// for all types that implement [`Format`].
#[sealed]
pub trait ErasedFormat: Send + Sync {
    /// Returns the primary media type for this format.
    fn primary_media_type(&self) -> MediaType<'static>;

    /// Returns the `Content-Type` header value for this format.
    fn content_type_header(&self) -> HeaderValue;

    /// Determines how specifically this format matches the requested media type.
    fn match_specificity(&self, requested: &MediaType<'_>) -> Option<MatchSpecificity>;

    /// Serializes data using a callback that receives a type-erased serializer.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    fn serialize(
        &self,
        bytes: &mut Vec<u8>,
        body: &mut dyn FnMut(&mut dyn Serializer) -> erased_serde::Result<()>,
    ) -> erased_serde::Result<()>;

    /// Deserializes data using a callback that receives a type-erased deserializer.
    ///
    /// # Errors
    ///
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
        let mut serializer = <F as Format>::serializer(self, bytes)?;
        let mut erased = <dyn Serializer>::erase(serializer.as_serializer());
        body(&mut erased)
    }

    fn deserialize(
        &self,
        bytes: &[u8],
        body: &mut dyn FnMut(&mut dyn Deserializer<'_>) -> erased_serde::Result<()>,
    ) -> erased_serde::Result<()> {
        let mut deserializer = <F as Format>::deserializer(self, bytes)?;
        let mut erased = <dyn Deserializer<'_>>::erase(deserializer.as_deserializer());
        body(&mut erased)
    }
}

impl Debug for dyn ErasedFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "ErasedFormat({})", self.primary_media_type())
    }
}
