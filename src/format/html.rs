//! HTML format implementation.

use http::HeaderValue;
use mediatype::MediaType;

use super::plain_text::{PlainTextDeserializer, PlainTextSerializer};
use super::{Borrowable, Format, OwnedDeserializer, OwnedSerializer};

/// HTML format (`text/html; charset=utf-8`).
///
/// This format only supports `String` values - serialization of other types will fail.
/// It reuses the same serializer/deserializer as [`super::PlainTextFormat`].
#[derive(Debug, Clone, Copy, Default)]
pub struct HtmlFormat;

impl Format for HtmlFormat {
    fn media_types(&self) -> &'static [MediaType<'static>] {
        static TYPES: &[MediaType<'_>] = &[mediatype::media_type!(TEXT / HTML)];
        TYPES
    }

    fn content_type_header(&self) -> HeaderValue {
        HeaderValue::from_static("text/html; charset=utf-8")
    }

    fn serializer<'a>(
        &'a self,
        bytes: &'a mut Vec<u8>,
    ) -> erased_serde::Result<impl OwnedSerializer + 'a> {
        Ok(PlainTextSerializer { output: bytes })
    }

    fn deserializer<'a>(
        &'a self,
        bytes: &'a [u8],
    ) -> erased_serde::Result<impl OwnedDeserializer<'a> + 'a> {
        Ok(Borrowable(PlainTextDeserializer { input: bytes }))
    }
}
