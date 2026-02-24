//! Plain text format implementation.

use http::HeaderValue;
use mediatype::MediaType;
use serde::{
    Deserializer,
    de::{self, Visitor},
    ser::{self, Impossible},
};

use super::{Borrowable, Format, OwnedDeserializer, OwnedSerializer};

/// Plain text format (`text/plain; charset=utf-8`).
///
/// This format only supports `String` values - serialization of other types will fail.
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

/// Serializer that writes string content directly as bytes.
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

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        Err(PlainTextError::UnsupportedType("bool"))
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        Err(PlainTextError::UnsupportedType("i8"))
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        Err(PlainTextError::UnsupportedType("i16"))
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        Err(PlainTextError::UnsupportedType("i32"))
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        Err(PlainTextError::UnsupportedType("i64"))
    }

    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
        Err(PlainTextError::UnsupportedType("u8"))
    }

    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        Err(PlainTextError::UnsupportedType("u16"))
    }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        Err(PlainTextError::UnsupportedType("u32"))
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        Err(PlainTextError::UnsupportedType("u64"))
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        Err(PlainTextError::UnsupportedType("f32"))
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Err(PlainTextError::UnsupportedType("f64"))
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        Err(PlainTextError::UnsupportedType("char"))
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(PlainTextError::UnsupportedType("bytes"))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(PlainTextError::UnsupportedType("none"))
    }

    fn serialize_some<T>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        Err(PlainTextError::UnsupportedType("some"))
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(PlainTextError::UnsupportedType("unit"))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(PlainTextError::UnsupportedType("unit struct"))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(PlainTextError::UnsupportedType("unit variant"))
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        Err(PlainTextError::UnsupportedType("newtype variant"))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(PlainTextError::UnsupportedType("sequence"))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(PlainTextError::UnsupportedType("tuple"))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(PlainTextError::UnsupportedType("tuple struct"))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(PlainTextError::UnsupportedType("tuple variant"))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(PlainTextError::UnsupportedType("map"))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(PlainTextError::UnsupportedType("struct"))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(PlainTextError::UnsupportedType("struct variant"))
    }
}

/// Deserializer that reads bytes as a UTF-8 string.
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

    fn deserialize_bool<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PlainTextError::UnsupportedType("bool"))
    }

    fn deserialize_i8<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PlainTextError::UnsupportedType("i8"))
    }

    fn deserialize_i16<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PlainTextError::UnsupportedType("i16"))
    }

    fn deserialize_i32<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PlainTextError::UnsupportedType("i32"))
    }

    fn deserialize_i64<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PlainTextError::UnsupportedType("i64"))
    }

    fn deserialize_u8<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PlainTextError::UnsupportedType("u8"))
    }

    fn deserialize_u16<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PlainTextError::UnsupportedType("u16"))
    }

    fn deserialize_u32<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PlainTextError::UnsupportedType("u32"))
    }

    fn deserialize_u64<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PlainTextError::UnsupportedType("u64"))
    }

    fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PlainTextError::UnsupportedType("f32"))
    }

    fn deserialize_f64<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PlainTextError::UnsupportedType("f64"))
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PlainTextError::UnsupportedType("char"))
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PlainTextError::UnsupportedType("bytes"))
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PlainTextError::UnsupportedType("byte_buf"))
    }

    fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PlainTextError::UnsupportedType("option"))
    }

    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PlainTextError::UnsupportedType("unit"))
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PlainTextError::UnsupportedType("unit struct"))
    }

    fn deserialize_seq<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PlainTextError::UnsupportedType("sequence"))
    }

    fn deserialize_tuple<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PlainTextError::UnsupportedType("tuple"))
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PlainTextError::UnsupportedType("tuple struct"))
    }

    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PlainTextError::UnsupportedType("map"))
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PlainTextError::UnsupportedType("struct"))
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PlainTextError::UnsupportedType("enum"))
    }

    fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PlainTextError::UnsupportedType("identifier"))
    }

    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PlainTextError::UnsupportedType("ignored_any"))
    }
}

/// Error type for plain text serialization/deserialization.
#[derive(Debug)]
pub enum PlainTextError {
    /// The type is not supported for plain text serialization.
    UnsupportedType(&'static str),
    /// The input bytes are not valid UTF-8.
    InvalidUtf8,
    /// A custom error message from serde.
    Custom(String),
}

impl std::fmt::Display for PlainTextError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedType(ty) => {
                write!(f, "plain text format only supports strings, not {ty}")
            }
            Self::InvalidUtf8 => write!(f, "input is not valid UTF-8"),
            Self::Custom(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for PlainTextError {}

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
