//! `MessagePack` format.

use http::HeaderValue;
use mediatype::MediaType;

use super::{Borrowable, Format, OwnedDeserializer, OwnedSerializer};

const MEDIA_TYPES: &[MediaType<'_>] = &[
    mediatype::media_type!(APPLICATION / MSGPACK),
    mediatype::media_type!(APPLICATION / x_::MSGPACK),
];

const CONTENT_TYPE: HeaderValue = HeaderValue::from_static("application/msgpack");

/// `MessagePack` format using compact (positional/array) serialization (`application/msgpack`).
///
/// Structs are serialized as arrays with values in field-declaration order. This is the
/// `rmp_serde` default and produces smaller payloads, but is incompatible with
/// `#[serde(tag = "...")]` and `#[serde(skip_serializing_if)]` on non-trailing fields.
///
/// Use [`MsgPackNamedFormat`] if your types require named (map) serialization.
#[derive(Debug, Clone, Copy, Default)]
pub struct MsgPackFormat;

impl Format for MsgPackFormat {
    fn media_types(&self) -> &'static [MediaType<'static>] {
        MEDIA_TYPES
    }

    fn content_type_header(&self) -> HeaderValue {
        CONTENT_TYPE
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

/// `MessagePack` format using named (map) serialization (`application/msgpack`).
///
/// Structs are serialized as maps with field names as keys. This is compatible with all serde
/// attributes (`#[serde(tag = "...")]`, `#[serde(skip_serializing_if)]`, etc.) at the cost
/// of slightly larger payloads.
#[derive(Debug, Clone, Copy, Default)]
pub struct MsgPackNamedFormat;

impl Format for MsgPackNamedFormat {
    fn media_types(&self) -> &'static [MediaType<'static>] {
        MEDIA_TYPES
    }

    fn content_type_header(&self) -> HeaderValue {
        CONTENT_TYPE
    }

    fn serializer<'a>(
        &'a self,
        bytes: &'a mut Vec<u8>,
    ) -> erased_serde::Result<impl OwnedSerializer + 'a> {
        Ok(rmp_serde::Serializer::new(bytes).with_struct_map())
    }

    fn deserializer<'a>(
        &'a self,
        bytes: &'a [u8],
    ) -> erased_serde::Result<impl OwnedDeserializer<'a> + 'a> {
        Ok(Borrowable(rmp_serde::Deserializer::from_read_ref(bytes)))
    }
}
