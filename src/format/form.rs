//! URL-encoded form format implementation.

use http::HeaderValue;
use mediatype::MediaType;
use serde::de::{self, IntoDeserializer, Visitor};
use serde::ser::{self, Impossible};
use serde::Deserializer;
use std::cell::RefCell;
use std::rc::Rc;

use super::{Format, OwnedDeserializer, OwnedSerializer};

/// URL-encoded form format (`application/x-www-form-urlencoded`).
///
/// This format supports flat structs and maps for serialization, and any
/// type supported by `serde_urlencoded` for deserialization.
#[derive(Debug, Clone, Copy, Default)]
pub struct FormFormat;

impl Format for FormFormat {
    fn media_types(&self) -> &'static [MediaType<'static>] {
        static TYPES: &[MediaType<'_>] = &[MediaType::new(
            mediatype::names::APPLICATION,
            mediatype::Name::new_unchecked("x-www-form-urlencoded"),
        )];
        TYPES
    }

    fn content_type_header(&self) -> HeaderValue {
        HeaderValue::from_static("application/x-www-form-urlencoded")
    }

    fn serializer<'a>(
        &'a self,
        bytes: &'a mut Vec<u8>,
    ) -> erased_serde::Result<impl OwnedSerializer + 'a> {
        Ok(FormSerializerWrapper::new(bytes))
    }

    fn deserializer<'a>(
        &'a self,
        bytes: &'a [u8],
    ) -> erased_serde::Result<impl OwnedDeserializer<'a> + 'a> {
        Ok(FormDeserializerWrapper::new(bytes))
    }
}

/// Shared state for form serialization.
struct SharedFormState {
    pairs: Vec<(String, String)>,
}

/// Custom serializer wrapper that handles String-to-bytes conversion.
pub(crate) struct FormSerializerWrapper<'out> {
    output: &'out mut Vec<u8>,
    state: Rc<RefCell<SharedFormState>>,
}

impl<'out> FormSerializerWrapper<'out> {
    fn new(output: &'out mut Vec<u8>) -> Self {
        Self {
            output,
            state: Rc::new(RefCell::new(SharedFormState { pairs: Vec::new() })),
        }
    }

    fn flush(&mut self) {
        let state = self.state.borrow();
        if state.pairs.is_empty() {
            return;
        }

        let mut target = String::new();
        {
            let mut serializer = form_urlencoded::Serializer::new(&mut target);
            for (key, value) in &state.pairs {
                serializer.append_pair(key, value);
            }
            serializer.finish();
        }
        self.output.extend_from_slice(target.as_bytes());
    }
}

impl Drop for FormSerializerWrapper<'_> {
    fn drop(&mut self) {
        self.flush();
    }
}

/// Struct serializer for form-encoded data.
pub(crate) struct FormStructSerializer {
    state: Rc<RefCell<SharedFormState>>,
}

/// Map serializer for form-encoded data.
pub(crate) struct FormMapSerializer {
    state: Rc<RefCell<SharedFormState>>,
    key: Option<String>,
}

impl ser::Serializer for &mut FormSerializerWrapper<'_> {
    type Ok = ();
    type Error = FormError;
    type SerializeSeq = Impossible<(), FormError>;
    type SerializeTuple = Impossible<(), FormError>;
    type SerializeTupleStruct = Impossible<(), FormError>;
    type SerializeTupleVariant = Impossible<(), FormError>;
    type SerializeMap = FormMapSerializer;
    type SerializeStruct = FormStructSerializer;
    type SerializeStructVariant = Impossible<(), FormError>;

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(FormStructSerializer {
            state: Rc::clone(&self.state),
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(FormMapSerializer {
            state: Rc::clone(&self.state),
            key: None,
        })
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

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        Err(FormError::UnsupportedType("bool"))
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        Err(FormError::UnsupportedType("i8"))
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        Err(FormError::UnsupportedType("i16"))
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        Err(FormError::UnsupportedType("i32"))
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        Err(FormError::UnsupportedType("i64"))
    }

    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
        Err(FormError::UnsupportedType("u8"))
    }

    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        Err(FormError::UnsupportedType("u16"))
    }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        Err(FormError::UnsupportedType("u32"))
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        Err(FormError::UnsupportedType("u64"))
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        Err(FormError::UnsupportedType("f32"))
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Err(FormError::UnsupportedType("f64"))
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        Err(FormError::UnsupportedType("char"))
    }

    fn serialize_str(self, _v: &str) -> Result<Self::Ok, Self::Error> {
        Err(FormError::UnsupportedType("str"))
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(FormError::UnsupportedType("bytes"))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(FormError::UnsupportedType("none"))
    }

    fn serialize_some<T>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        Err(FormError::UnsupportedType("some"))
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(FormError::UnsupportedType("unit"))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(FormError::UnsupportedType("unit struct"))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(FormError::UnsupportedType("unit variant"))
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
        Err(FormError::UnsupportedType("newtype variant"))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(FormError::UnsupportedType("sequence"))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(FormError::UnsupportedType("tuple"))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(FormError::UnsupportedType("tuple struct"))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(FormError::UnsupportedType("tuple variant"))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(FormError::UnsupportedType("struct variant"))
    }
}

impl ser::SerializeStruct for FormStructSerializer {
    type Ok = ();
    type Error = FormError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        let value_str = value_to_string(value)?;
        self.state
            .borrow_mut()
            .pairs
            .push((key.to_string(), value_str));
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl ser::SerializeMap for FormMapSerializer {
    type Ok = ();
    type Error = FormError;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.key = Some(value_to_string(key)?);
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        let key = self
            .key
            .take()
            .ok_or_else(|| FormError::Custom("serialize_value called without key".to_string()))?;
        let value_str = value_to_string(value)?;
        self.state.borrow_mut().pairs.push((key, value_str));
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

/// Serialize a value to a string for form encoding.
#[allow(clippy::too_many_lines)]
fn value_to_string<T: ?Sized + serde::Serialize>(value: &T) -> Result<String, FormError> {
    struct StringSerializer;

    impl ser::Serializer for StringSerializer {
        type Ok = String;
        type Error = FormError;
        type SerializeSeq = Impossible<String, FormError>;
        type SerializeTuple = Impossible<String, FormError>;
        type SerializeTupleStruct = Impossible<String, FormError>;
        type SerializeTupleVariant = Impossible<String, FormError>;
        type SerializeMap = Impossible<String, FormError>;
        type SerializeStruct = Impossible<String, FormError>;
        type SerializeStructVariant = Impossible<String, FormError>;

        fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
            Ok(v.to_string())
        }

        fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
            Ok(v.to_string())
        }

        fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
            Ok(v.to_string())
        }

        fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
            Ok(v.to_string())
        }

        fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
            Ok(v.to_string())
        }

        fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
            Ok(v.to_string())
        }

        fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
            Ok(v.to_string())
        }

        fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
            Ok(v.to_string())
        }

        fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
            Ok(v.to_string())
        }

        fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
            Ok(v.to_string())
        }

        fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
            Ok(v.to_string())
        }

        fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
            Ok(v.to_string())
        }

        fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
            Ok(v.to_string())
        }

        fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
            Err(FormError::UnsupportedType("bytes in field value"))
        }

        fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
            Ok(String::new())
        }

        fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
        where
            T: ?Sized + serde::Serialize,
        {
            value.serialize(self)
        }

        fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
            Ok(String::new())
        }

        fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
            Ok(String::new())
        }

        fn serialize_unit_variant(
            self,
            _name: &'static str,
            _variant_index: u32,
            variant: &'static str,
        ) -> Result<Self::Ok, Self::Error> {
            Ok(variant.to_string())
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
            Err(FormError::UnsupportedType("newtype variant in field value"))
        }

        fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
            Err(FormError::UnsupportedType("sequence in field value"))
        }

        fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
            Err(FormError::UnsupportedType("tuple in field value"))
        }

        fn serialize_tuple_struct(
            self,
            _name: &'static str,
            _len: usize,
        ) -> Result<Self::SerializeTupleStruct, Self::Error> {
            Err(FormError::UnsupportedType("tuple struct in field value"))
        }

        fn serialize_tuple_variant(
            self,
            _name: &'static str,
            _variant_index: u32,
            _variant: &'static str,
            _len: usize,
        ) -> Result<Self::SerializeTupleVariant, Self::Error> {
            Err(FormError::UnsupportedType("tuple variant in field value"))
        }

        fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
            Err(FormError::UnsupportedType("map in field value"))
        }

        fn serialize_struct(
            self,
            _name: &'static str,
            _len: usize,
        ) -> Result<Self::SerializeStruct, Self::Error> {
            Err(FormError::UnsupportedType("struct in field value"))
        }

        fn serialize_struct_variant(
            self,
            _name: &'static str,
            _variant_index: u32,
            _variant: &'static str,
            _len: usize,
        ) -> Result<Self::SerializeStructVariant, Self::Error> {
            Err(FormError::UnsupportedType("struct variant in field value"))
        }
    }

    value.serialize(StringSerializer)
}

/// Custom deserializer wrapper for form-encoded data.
///
/// Wraps the parsed form data and implements `Deserializer` for `&mut T`.
pub(crate) struct FormDeserializerWrapper<'de> {
    pairs: Vec<(String, String)>,
    _marker: std::marker::PhantomData<&'de ()>,
}

impl FormDeserializerWrapper<'_> {
    fn new(bytes: &[u8]) -> Self {
        let pairs: Vec<_> = form_urlencoded::parse(bytes)
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect();
        Self {
            pairs,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'de> Deserializer<'de> for &mut FormDeserializerWrapper<'de> {
    type Error = FormError;

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(FormMapAccess {
            pairs: &self.pairs,
            index: 0,
        })
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(FormMapAccess {
            pairs: &self.pairs,
            index: 0,
        })
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
        self.deserialize_map(visitor)
    }

    fn deserialize_bool<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(FormError::UnsupportedType("bool"))
    }

    fn deserialize_i8<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(FormError::UnsupportedType("i8"))
    }

    fn deserialize_i16<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(FormError::UnsupportedType("i16"))
    }

    fn deserialize_i32<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(FormError::UnsupportedType("i32"))
    }

    fn deserialize_i64<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(FormError::UnsupportedType("i64"))
    }

    fn deserialize_u8<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(FormError::UnsupportedType("u8"))
    }

    fn deserialize_u16<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(FormError::UnsupportedType("u16"))
    }

    fn deserialize_u32<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(FormError::UnsupportedType("u32"))
    }

    fn deserialize_u64<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(FormError::UnsupportedType("u64"))
    }

    fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(FormError::UnsupportedType("f32"))
    }

    fn deserialize_f64<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(FormError::UnsupportedType("f64"))
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(FormError::UnsupportedType("char"))
    }

    fn deserialize_str<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(FormError::UnsupportedType("str"))
    }

    fn deserialize_string<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(FormError::UnsupportedType("string"))
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(FormError::UnsupportedType("bytes"))
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(FormError::UnsupportedType("byte_buf"))
    }

    fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(FormError::UnsupportedType("option"))
    }

    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(FormError::UnsupportedType("unit"))
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(FormError::UnsupportedType("unit struct"))
    }

    fn deserialize_seq<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(FormError::UnsupportedType("sequence"))
    }

    fn deserialize_tuple<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(FormError::UnsupportedType("tuple"))
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
        Err(FormError::UnsupportedType("tuple struct"))
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
        Err(FormError::UnsupportedType("enum"))
    }

    fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(FormError::UnsupportedType("identifier"))
    }

    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(FormError::UnsupportedType("ignored_any"))
    }
}

/// Map access implementation for form deserialization.
struct FormMapAccess<'a> {
    pairs: &'a [(String, String)],
    index: usize,
}

impl<'de> de::MapAccess<'de> for FormMapAccess<'_> {
    type Error = FormError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.index >= self.pairs.len() {
            return Ok(None);
        }
        let key = &self.pairs[self.index].0;
        seed.deserialize(FormValueDeserializer(key)).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let value = &self.pairs[self.index].1;
        self.index += 1;
        seed.deserialize(FormValueDeserializer(value))
    }
}

/// Simple deserializer for individual form values (strings that may represent other types).
struct FormValueDeserializer<'a>(&'a str);

impl<'de> Deserializer<'de> for FormValueDeserializer<'_> {
    type Error = FormError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_str(self.0)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_str(self.0)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(self.0.to_owned())
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.0 {
            "true" | "1" => visitor.visit_bool(true),
            "false" | "0" => visitor.visit_bool(false),
            _ => Err(FormError::Custom(format!(
                "invalid boolean value: {}",
                self.0
            ))),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let v = self
            .0
            .parse()
            .map_err(|e| FormError::Custom(format!("invalid i8: {e}")))?;
        visitor.visit_i8(v)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let v = self
            .0
            .parse()
            .map_err(|e| FormError::Custom(format!("invalid i16: {e}")))?;
        visitor.visit_i16(v)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let v = self
            .0
            .parse()
            .map_err(|e| FormError::Custom(format!("invalid i32: {e}")))?;
        visitor.visit_i32(v)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let v = self
            .0
            .parse()
            .map_err(|e| FormError::Custom(format!("invalid i64: {e}")))?;
        visitor.visit_i64(v)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let v = self
            .0
            .parse()
            .map_err(|e| FormError::Custom(format!("invalid u8: {e}")))?;
        visitor.visit_u8(v)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let v = self
            .0
            .parse()
            .map_err(|e| FormError::Custom(format!("invalid u16: {e}")))?;
        visitor.visit_u16(v)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let v = self
            .0
            .parse()
            .map_err(|e| FormError::Custom(format!("invalid u32: {e}")))?;
        visitor.visit_u32(v)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let v = self
            .0
            .parse()
            .map_err(|e| FormError::Custom(format!("invalid u64: {e}")))?;
        visitor.visit_u64(v)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let v = self
            .0
            .parse()
            .map_err(|e| FormError::Custom(format!("invalid f32: {e}")))?;
        visitor.visit_f32(v)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let v = self
            .0
            .parse()
            .map_err(|e| FormError::Custom(format!("invalid f64: {e}")))?;
        visitor.visit_f64(v)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut chars = self.0.chars();
        let c = chars
            .next()
            .ok_or_else(|| FormError::Custom("expected char, got empty string".to_string()))?;
        if chars.next().is_some() {
            return Err(FormError::Custom(format!(
                "expected single char, got: {}",
                self.0
            )));
        }
        visitor.visit_char(c)
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(FormError::UnsupportedType("bytes"))
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(FormError::UnsupportedType("byte_buf"))
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.0.is_empty() {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
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

    fn deserialize_seq<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(FormError::UnsupportedType("sequence in value"))
    }

    fn deserialize_tuple<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(FormError::UnsupportedType("tuple in value"))
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
        Err(FormError::UnsupportedType("tuple struct in value"))
    }

    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(FormError::UnsupportedType("map in value"))
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
        Err(FormError::UnsupportedType("struct in value"))
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_enum(self.0.to_owned().into_deserializer())
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_str(self.0)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }
}

/// Error type for form serialization/deserialization.
#[derive(Debug)]
pub enum FormError {
    /// The type is not supported for form serialization.
    UnsupportedType(&'static str),
    /// A custom error message from serde.
    Custom(String),
}

impl std::fmt::Display for FormError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedType(ty) => {
                write!(f, "form encoding does not support {ty}")
            }
            Self::Custom(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for FormError {}

impl ser::Error for FormError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Self::Custom(msg.to_string())
    }
}

impl de::Error for FormError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Self::Custom(msg.to_string())
    }
}
