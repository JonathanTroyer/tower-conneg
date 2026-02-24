//! BSON format.

use bson::{Bson, Document, RawDocumentBuf};
use erased_serde::Serializer as ErasedSerializer;
use http::HeaderValue;
use mediatype::{MediaType, Name, names::APPLICATION};

use super::{Consumable, Format, OwnedDeserializer, OwnedSerializer};

/// BSON format (`application/bson`).
#[derive(Debug, Clone, Copy, Default)]
pub struct BsonFormat;

impl Format for BsonFormat {
    fn media_types(&self) -> &'static [MediaType<'static>] {
        static TYPES: &[MediaType<'_>] =
            &[MediaType::new(APPLICATION, Name::new_unchecked("bson"))];
        TYPES
    }

    fn content_type_header(&self) -> HeaderValue {
        HeaderValue::from_static("application/bson")
    }

    fn serializer<'a>(
        &'a self,
        bytes: &'a mut Vec<u8>,
    ) -> erased_serde::Result<impl OwnedSerializer + 'a> {
        Ok(BsonOwnedSerializer(bytes))
    }

    fn deserializer<'a>(
        &'a self,
        bytes: &'a [u8],
    ) -> erased_serde::Result<impl OwnedDeserializer<'a> + 'a> {
        let raw_doc =
            RawDocumentBuf::from_bytes(bytes.to_vec()).map_err(serde::de::Error::custom)?;
        let doc = Document::try_from(raw_doc).map_err(serde::de::Error::custom)?;
        Ok(Consumable::new(bson::Deserializer::new(Bson::Document(
            doc,
        ))))
    }
}

struct BsonOwnedSerializer<'a>(&'a mut Vec<u8>);

impl OwnedSerializer for BsonOwnedSerializer<'_> {
    fn with_erased(
        self,
        f: &mut dyn FnMut(&mut dyn ErasedSerializer) -> erased_serde::Result<()>,
    ) -> erased_serde::Result<()> {
        // BSON's serializer is consuming and returns Bson as its Ok type,
        // which erased-serde discards. We use JSON as an intermediate format.
        let mut json_bytes = Vec::new();
        {
            let mut json_ser = serde_json::Serializer::new(&mut json_bytes);
            let mut erased = <dyn ErasedSerializer>::erase(&mut json_ser);
            f(&mut erased)?;
        }

        // Parse JSON and convert to BSON document
        let json_value: serde_json::Value =
            serde_json::from_slice(&json_bytes).map_err(serde::ser::Error::custom)?;

        // Serialize to BSON document and write bytes
        let doc = bson::serialize_to_document(&json_value).map_err(serde::ser::Error::custom)?;
        doc.to_writer(self.0).map_err(serde::ser::Error::custom)?;

        Ok(())
    }
}
