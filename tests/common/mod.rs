#![allow(elided_lifetimes_in_paths)]

use http::HeaderValue;
use mediatype::MediaType;
use tower_conneg::{Format, OwnedDeserializer, OwnedSerializer};

pub(crate) struct JsonFormat;

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

/// A mock XML format for testing. Uses JSON serialization internally but reports
/// application/xml media type for content negotiation testing purposes.
pub(crate) struct XmlFormat;

impl Format for XmlFormat {
    fn media_types(&self) -> &'static [MediaType<'static>] {
        static TYPES: &[MediaType<'_>] = &[mediatype::media_type!(APPLICATION / XML)];
        TYPES
    }

    fn content_type_header(&self) -> HeaderValue {
        HeaderValue::from_static("application/xml")
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
