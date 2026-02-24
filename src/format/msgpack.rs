//! `MessagePack` format implementation.

use http::HeaderValue;
use mediatype::{MediaType, Name, names::APPLICATION};

use super::{Borrowable, Format, OwnedDeserializer, OwnedSerializer};

/// `MessagePack` serialization format using `rmp-serde`.
#[derive(Debug, Clone, Copy, Default)]
pub struct MsgPackFormat;

impl Format for MsgPackFormat {
    fn media_types(&self) -> &'static [MediaType<'static>] {
        static TYPES: &[MediaType<'_>] = &[
            MediaType::new(APPLICATION, Name::new_unchecked("msgpack")),
            MediaType::new(APPLICATION, Name::new_unchecked("x-msgpack")),
        ];
        TYPES
    }

    fn content_type_header(&self) -> HeaderValue {
        HeaderValue::from_static("application/msgpack")
    }

    fn serializer<'a>(
        &'a self,
        bytes: &'a mut Vec<u8>,
    ) -> erased_serde::Result<impl OwnedSerializer + 'a> {
        Ok(rmp_serde::Serializer::new(bytes))
    }

    fn deserializer<'a>(
        &'a self,
        bytes: &'a [u8],
    ) -> erased_serde::Result<impl OwnedDeserializer<'a> + 'a> {
        Ok(Borrowable(rmp_serde::Deserializer::from_read_ref(bytes)))
    }
}
