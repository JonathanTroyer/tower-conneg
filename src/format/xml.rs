//! XML format.

use erased_serde::Serializer as ErasedSerializer;
use http::HeaderValue;
use mediatype::MediaType;

use super::{Borrowable, Format, OwnedDeserializer, OwnedSerializer};

/// XML format (`application/xml`).
#[derive(Debug, Clone, Copy, Default)]
pub struct XmlFormat;

impl Format for XmlFormat {
    fn media_types(&self) -> &'static [MediaType<'static>] {
        static TYPES: &[MediaType<'_>] = &[
            mediatype::media_type!(APPLICATION / XML),
            mediatype::media_type!(TEXT / XML),
        ];
        TYPES
    }

    fn content_type_header(&self) -> HeaderValue {
        HeaderValue::from_static("application/xml")
    }

    fn serializer<'a>(
        &'a self,
        bytes: &'a mut Vec<u8>,
    ) -> erased_serde::Result<impl OwnedSerializer + 'a> {
        Ok(XmlOwnedSerializer(bytes))
    }

    fn deserializer<'a>(
        &'a self,
        bytes: &'a [u8],
    ) -> erased_serde::Result<impl OwnedDeserializer<'a> + 'a> {
        let s = std::str::from_utf8(bytes).map_err(serde::de::Error::custom)?;
        Ok(Borrowable(quick_xml::de::Deserializer::from_str(s)))
    }
}

struct XmlOwnedSerializer<'a>(&'a mut Vec<u8>);

impl OwnedSerializer for XmlOwnedSerializer<'_> {
    fn with_erased(
        self,
        f: &mut dyn FnMut(&mut dyn ErasedSerializer) -> erased_serde::Result<()>,
    ) -> erased_serde::Result<()> {
        let mut writer = FmtWrite(self.0);
        let serializer = quick_xml::se::Serializer::new(&mut writer);
        let mut erased = <dyn ErasedSerializer>::erase(serializer);
        f(&mut erased)
    }
}

struct FmtWrite<'a>(&'a mut Vec<u8>);

impl std::fmt::Write for FmtWrite<'_> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.0.extend_from_slice(s.as_bytes());
        Ok(())
    }
}
