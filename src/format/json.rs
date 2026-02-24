//! JSON format implementation.

use http::HeaderValue;
use mediatype::MediaType;

use super::{Format, OwnedDeserializer, OwnedSerializer};

/// JSON serialization format using `serde_json`.
#[derive(Debug, Clone, Copy, Default)]
pub struct JsonFormat;

impl Format for JsonFormat {
    fn media_types(&self) -> &'static [MediaType<'static>] {
        static TYPES: &[MediaType<'_>] = &[mediatype::media_type!(APPLICATION / JSON)];
        TYPES
    }

    fn content_type_header(&self) -> HeaderValue {
        HeaderValue::from_static("application/json")
    }

    fn serializer<'a>(
        &'a self,
        bytes: &'a mut Vec<u8>,
    ) -> erased_serde::Result<impl OwnedSerializer + 'a> {
        Ok(serde_json::Serializer::new(bytes))
    }

    fn deserializer<'a>(
        &'a self,
        bytes: &'a [u8],
    ) -> erased_serde::Result<impl OwnedDeserializer<'a> + 'a> {
        Ok(serde_json::Deserializer::from_slice(bytes))
    }
}
