//! Plain text format.

use http::HeaderValue;
use mediatype::MediaType;
use serde::{
    Deserializer,
    de::{self, Visitor},
    ser::{self, Impossible},
};

use super::{Borrowable, Format, OwnedDeserializer, OwnedSerializer};

/// Plain text format (`text/plain`). Only supports `String` values.
#[derive(Debug, Clone, Copy, Default)]
pub struct PlainTextFormat;

impl Format for PlainTextFormat {
    fn media_types(&self) -> &'static [MediaType<'static>] {
        static TYPES: &[MediaType<'_>] = &[mediatype::media_type!(TEXT / PLAIN)];
        TYPES
    }

    fn content_type_header(&self) -> HeaderValue {
        HeaderValue::from_static("text/plain; charset=utf-8")
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

pub(crate) struct PlainTextSerializer<'out> {
    pub(crate) output: &'out mut Vec<u8>,
}

impl ser::Serializer for &mut PlainTextSerializer<'_> {
    type Ok = ();
    type Error = PlainTextError;
    type SerializeSeq = Impossible<(), PlainTextError>;
    type SerializeTuple = Impossible<(), PlainTextError>;
    type SerializeTupleStruct = Impossible<(), PlainTextError>;
    type SerializeTupleVariant = Impossible<(), PlainTextError>;
    type SerializeMap = Impossible<(), PlainTextError>;
    type SerializeStruct = Impossible<(), PlainTextError>;
    type SerializeStructVariant = Impossible<(), PlainTextError>;

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.output.extend_from_slice(v.as_bytes());
        Ok(())
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    reject_serializer_types!(PlainTextError => {
        primitives: [bool, i8, i16, i32, i64, u8, u16, u32, u64, f32, f64, char, bytes, none, some, unit]
        compound: [unit_struct, unit_variant, newtype_variant, seq, tuple, tuple_struct, tuple_variant, map, struct_, struct_variant]
    });
}

pub(crate) struct PlainTextDeserializer<'de> {
    pub(crate) input: &'de [u8],
}

impl<'de> Deserializer<'de> for &mut PlainTextDeserializer<'de> {
    type Error = PlainTextError;

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let s = std::str::from_utf8(self.input).map_err(|_| PlainTextError::InvalidUtf8)?;
        visitor.visit_borrowed_str(s)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    reject_deserializer_types!(PlainTextError => {
        primitives: [bool, i8, i16, i32, i64, u8, u16, u32, u64, f32, f64, char, bytes, byte_buf, option, unit, identifier, ignored_any]
        compound: [unit_struct, seq, tuple, tuple_struct, map, struct_, enum_]
    });
}

/// Plain text serialization error.
#[derive(Debug, thiserror::Error)]
pub enum PlainTextError {
    #[error("plain text format only supports strings, not {0}")]
    UnsupportedType(&'static str),
    #[error("input is not valid UTF-8")]
    InvalidUtf8,
    #[error("{0}")]
    Custom(String),
}

impl ser::Error for PlainTextError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Self::Custom(msg.to_string())
    }
}

impl de::Error for PlainTextError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Self::Custom(msg.to_string())
    }
}
