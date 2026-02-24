//! URL-encoded form format.

use http::HeaderValue;
use mediatype::MediaType;
use serde::de;
use serde::ser::{self, Impossible};
use std::cell::RefCell;
use std::rc::Rc;

use super::{Consumable, Format, OwnedDeserializer, OwnedSerializer};

/// URL-encoded form format (`application/x-www-form-urlencoded`).
///
/// Supports flat structs and maps for serialization.
#[derive(Debug, Clone, Copy, Default)]
pub struct FormFormat;

impl Format for FormFormat {
    fn media_types(&self) -> &'static [MediaType<'static>] {
        static TYPES: &[MediaType<'_>] = &[mediatype::media_type!(
            APPLICATION / x_::WWW_FORM_URLENCODED
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
        let inner = serde_urlencoded::Deserializer::new(form_urlencoded::parse(bytes));
        Ok(Consumable::new(inner))
    }
}

struct SharedFormState {
    pairs: Vec<(String, String)>,
}

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

pub(crate) struct FormStructSerializer {
    state: Rc<RefCell<SharedFormState>>,
}

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

    reject_serializer_types!(FormError => {
        primitives: [bool, i8, i16, i32, i64, u8, u16, u32, u64, f32, f64, char, str, bytes, none, some, unit]
        compound: [unit_struct, unit_variant, newtype_variant, seq, tuple, tuple_struct, tuple_variant, struct_variant]
    });
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

/// Form serialization error.
#[derive(Debug, thiserror::Error)]
pub enum FormError {
    #[error("form encoding does not support {0}")]
    UnsupportedType(&'static str),
    #[error("{0}")]
    Custom(String),
}

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
