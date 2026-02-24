//! Postcard format.

use erased_serde::Serializer as ErasedSerializer;
use http::HeaderValue;
use mediatype::{MediaType, Name, names::APPLICATION};
use postcard::ser_flavors::{AllocVec, Flavor};

use super::{Borrowable, Format, OwnedDeserializer, OwnedSerializer};

/// Postcard format (`application/x-postcard`).
#[derive(Debug, Clone, Copy, Default)]
pub struct PostcardFormat;

impl Format for PostcardFormat {
    fn media_types(&self) -> &'static [MediaType<'static>] {
        static TYPES: &[MediaType<'_>] = &[MediaType::new(
            APPLICATION,
            Name::new_unchecked("x-postcard"),
        )];
        TYPES
    }

    fn content_type_header(&self) -> HeaderValue {
        HeaderValue::from_static("application/x-postcard")
    }

    fn serializer<'a>(
        &'a self,
        bytes: &'a mut Vec<u8>,
    ) -> erased_serde::Result<impl OwnedSerializer + 'a> {
        Ok(PostcardOwnedSerializer(bytes))
    }

    fn deserializer<'a>(
        &'a self,
        bytes: &'a [u8],
    ) -> erased_serde::Result<impl OwnedDeserializer<'a> + 'a> {
        Ok(Borrowable(postcard::Deserializer::from_bytes(bytes)))
    }
}

struct PostcardOwnedSerializer<'a>(&'a mut Vec<u8>);

impl OwnedSerializer for PostcardOwnedSerializer<'_> {
    fn with_erased(
        self,
        f: &mut dyn FnMut(&mut dyn ErasedSerializer) -> erased_serde::Result<()>,
    ) -> erased_serde::Result<()> {
        let mut serializer = postcard::Serializer {
            output: AllocVec::new(),
        };
        let mut erased = <dyn ErasedSerializer>::erase(&mut serializer);
        f(&mut erased)?;
        let result = serializer
            .output
            .finalize()
            .map_err(serde::ser::Error::custom)?;
        self.0.extend_from_slice(&result);
        Ok(())
    }
}
