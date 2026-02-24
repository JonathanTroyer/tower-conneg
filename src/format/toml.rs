//! TOML format.

use erased_serde::Serializer as ErasedSerializer;
use http::HeaderValue;
use mediatype::{MediaType, Name, names::APPLICATION};

use super::{Consumable, Format, OwnedDeserializer, OwnedSerializer};

/// TOML format (`application/toml`).
#[derive(Debug, Clone, Copy, Default)]
pub struct TomlFormat;

impl Format for TomlFormat {
    fn media_types(&self) -> &'static [MediaType<'static>] {
        static TYPES: &[MediaType<'_>] =
            &[MediaType::new(APPLICATION, Name::new_unchecked("toml"))];
        TYPES
    }

    fn content_type_header(&self) -> HeaderValue {
        HeaderValue::from_static("application/toml")
    }

    fn serializer<'a>(
        &'a self,
        bytes: &'a mut Vec<u8>,
    ) -> erased_serde::Result<impl OwnedSerializer + 'a> {
        Ok(TomlOwnedSerializer(bytes))
    }

    fn deserializer<'a>(
        &'a self,
        bytes: &'a [u8],
    ) -> erased_serde::Result<impl OwnedDeserializer<'a> + 'a> {
        let s = std::str::from_utf8(bytes).map_err(serde::de::Error::custom)?;
        Ok(Consumable::new(toml::de::Deserializer::new(s)))
    }
}

struct TomlOwnedSerializer<'a>(&'a mut Vec<u8>);

impl OwnedSerializer for TomlOwnedSerializer<'_> {
    fn with_erased(
        self,
        f: &mut dyn FnMut(&mut dyn ErasedSerializer) -> erased_serde::Result<()>,
    ) -> erased_serde::Result<()> {
        let mut string = String::new();
        let serializer = toml::ser::Serializer::new(&mut string);
        let mut erased = <dyn ErasedSerializer>::erase(serializer);
        f(&mut erased)?;
        self.0.extend_from_slice(string.as_bytes());
        Ok(())
    }
}
