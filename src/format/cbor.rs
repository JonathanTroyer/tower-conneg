//! CBOR format implementation.

use cbor4ii::core::utils::{IoWriter, SliceReader};
use http::HeaderValue;
use mediatype::{MediaType, Name, names::APPLICATION};

use super::{Borrowable, Format, OwnedDeserializer, OwnedSerializer};

/// CBOR serialization format using `cbor4ii`.
#[derive(Debug, Clone, Copy, Default)]
pub struct CborFormat;

impl Format for CborFormat {
    fn media_types(&self) -> &'static [MediaType<'static>] {
        static TYPES: &[MediaType<'_>] =
            &[MediaType::new(APPLICATION, Name::new_unchecked("cbor"))];
        TYPES
    }

    fn content_type_header(&self) -> HeaderValue {
        HeaderValue::from_static("application/cbor")
    }

    fn serializer<'a>(
        &'a self,
        bytes: &'a mut Vec<u8>,
    ) -> erased_serde::Result<impl OwnedSerializer + 'a> {
        Ok(cbor4ii::serde::Serializer::new(IoWriter::new(bytes)))
    }

    fn deserializer<'a>(
        &'a self,
        bytes: &'a [u8],
    ) -> erased_serde::Result<impl OwnedDeserializer<'a> + 'a> {
        Ok(Borrowable(cbor4ii::serde::Deserializer::new(
            SliceReader::new(bytes),
        )))
    }
}
