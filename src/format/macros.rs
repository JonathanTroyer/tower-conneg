//! Macros for reducing boilerplate in serde serializer/deserializer implementations.

/// Generates rejection methods for a serde `Serializer` that only supports specific types.
///
/// Each method returns `Err($error_type::UnsupportedType("type_name"))`.
macro_rules! reject_serializer_types {
    ($error_type:ty => {
        $(primitives: [$($primitive:ident),* $(,)?])?
        $(compound: [$($compound:ident),* $(,)?])?
    }) => {
        $($(
            reject_serializer_types!(@primitive $error_type, $primitive);
        )*)?
        $($(
            reject_serializer_types!(@compound $error_type, $compound);
        )*)?
    };

    (@primitive $error_type:ty, bool) => {
        fn serialize_bool(self, _: bool) -> Result<Self::Ok, Self::Error> {
            Err(<$error_type>::UnsupportedType("bool"))
        }
    };
    (@primitive $error_type:ty, i8) => {
        fn serialize_i8(self, _: i8) -> Result<Self::Ok, Self::Error> {
            Err(<$error_type>::UnsupportedType("i8"))
        }
    };
    (@primitive $error_type:ty, i16) => {
        fn serialize_i16(self, _: i16) -> Result<Self::Ok, Self::Error> {
            Err(<$error_type>::UnsupportedType("i16"))
        }
    };
    (@primitive $error_type:ty, i32) => {
        fn serialize_i32(self, _: i32) -> Result<Self::Ok, Self::Error> {
            Err(<$error_type>::UnsupportedType("i32"))
        }
    };
    (@primitive $error_type:ty, i64) => {
        fn serialize_i64(self, _: i64) -> Result<Self::Ok, Self::Error> {
            Err(<$error_type>::UnsupportedType("i64"))
        }
    };
    (@primitive $error_type:ty, u8) => {
        fn serialize_u8(self, _: u8) -> Result<Self::Ok, Self::Error> {
            Err(<$error_type>::UnsupportedType("u8"))
        }
    };
    (@primitive $error_type:ty, u16) => {
        fn serialize_u16(self, _: u16) -> Result<Self::Ok, Self::Error> {
            Err(<$error_type>::UnsupportedType("u16"))
        }
    };
    (@primitive $error_type:ty, u32) => {
        fn serialize_u32(self, _: u32) -> Result<Self::Ok, Self::Error> {
            Err(<$error_type>::UnsupportedType("u32"))
        }
    };
    (@primitive $error_type:ty, u64) => {
        fn serialize_u64(self, _: u64) -> Result<Self::Ok, Self::Error> {
            Err(<$error_type>::UnsupportedType("u64"))
        }
    };
    (@primitive $error_type:ty, f32) => {
        fn serialize_f32(self, _: f32) -> Result<Self::Ok, Self::Error> {
            Err(<$error_type>::UnsupportedType("f32"))
        }
    };
    (@primitive $error_type:ty, f64) => {
        fn serialize_f64(self, _: f64) -> Result<Self::Ok, Self::Error> {
            Err(<$error_type>::UnsupportedType("f64"))
        }
    };
    (@primitive $error_type:ty, char) => {
        fn serialize_char(self, _: char) -> Result<Self::Ok, Self::Error> {
            Err(<$error_type>::UnsupportedType("char"))
        }
    };
    (@primitive $error_type:ty, str) => {
        fn serialize_str(self, _: &str) -> Result<Self::Ok, Self::Error> {
            Err(<$error_type>::UnsupportedType("str"))
        }
    };
    (@primitive $error_type:ty, bytes) => {
        fn serialize_bytes(self, _: &[u8]) -> Result<Self::Ok, Self::Error> {
            Err(<$error_type>::UnsupportedType("bytes"))
        }
    };
    (@primitive $error_type:ty, none) => {
        fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
            Err(<$error_type>::UnsupportedType("none"))
        }
    };
    (@primitive $error_type:ty, some) => {
        fn serialize_some<T>(self, _: &T) -> Result<Self::Ok, Self::Error>
        where
            T: ?Sized + serde::Serialize,
        {
            Err(<$error_type>::UnsupportedType("some"))
        }
    };
    (@primitive $error_type:ty, unit) => {
        fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
            Err(<$error_type>::UnsupportedType("unit"))
        }
    };

    (@compound $error_type:ty, unit_struct) => {
        fn serialize_unit_struct(self, _: &'static str) -> Result<Self::Ok, Self::Error> {
            Err(<$error_type>::UnsupportedType("unit struct"))
        }
    };
    (@compound $error_type:ty, unit_variant) => {
        fn serialize_unit_variant(
            self,
            _: &'static str,
            _: u32,
            _: &'static str,
        ) -> Result<Self::Ok, Self::Error> {
            Err(<$error_type>::UnsupportedType("unit variant"))
        }
    };
    (@compound $error_type:ty, newtype_variant) => {
        fn serialize_newtype_variant<T>(
            self,
            _: &'static str,
            _: u32,
            _: &'static str,
            _: &T,
        ) -> Result<Self::Ok, Self::Error>
        where
            T: ?Sized + serde::Serialize,
        {
            Err(<$error_type>::UnsupportedType("newtype variant"))
        }
    };
    (@compound $error_type:ty, seq) => {
        fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
            Err(<$error_type>::UnsupportedType("sequence"))
        }
    };
    (@compound $error_type:ty, tuple) => {
        fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, Self::Error> {
            Err(<$error_type>::UnsupportedType("tuple"))
        }
    };
    (@compound $error_type:ty, tuple_struct) => {
        fn serialize_tuple_struct(
            self,
            _: &'static str,
            _: usize,
        ) -> Result<Self::SerializeTupleStruct, Self::Error> {
            Err(<$error_type>::UnsupportedType("tuple struct"))
        }
    };
    (@compound $error_type:ty, tuple_variant) => {
        fn serialize_tuple_variant(
            self,
            _: &'static str,
            _: u32,
            _: &'static str,
            _: usize,
        ) -> Result<Self::SerializeTupleVariant, Self::Error> {
            Err(<$error_type>::UnsupportedType("tuple variant"))
        }
    };
    (@compound $error_type:ty, map) => {
        fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
            Err(<$error_type>::UnsupportedType("map"))
        }
    };
    (@compound $error_type:ty, struct_) => {
        fn serialize_struct(
            self,
            _: &'static str,
            _: usize,
        ) -> Result<Self::SerializeStruct, Self::Error> {
            Err(<$error_type>::UnsupportedType("struct"))
        }
    };
    (@compound $error_type:ty, struct_variant) => {
        fn serialize_struct_variant(
            self,
            _: &'static str,
            _: u32,
            _: &'static str,
            _: usize,
        ) -> Result<Self::SerializeStructVariant, Self::Error> {
            Err(<$error_type>::UnsupportedType("struct variant"))
        }
    };
}

/// Generates rejection methods for a serde `Deserializer` that only supports specific types.
///
/// Each method returns `Err($error_type::UnsupportedType("type_name"))`.
macro_rules! reject_deserializer_types {
    ($error_type:ty => {
        $(primitives: [$($primitive:ident),* $(,)?])?
        $(compound: [$($compound:ident),* $(,)?])?
    }) => {
        $($(
            reject_deserializer_types!(@primitive $error_type, $primitive);
        )*)?
        $($(
            reject_deserializer_types!(@compound $error_type, $compound);
        )*)?
    };

    (@primitive $error_type:ty, bool) => {
        fn deserialize_bool<V>(self, _: V) -> Result<V::Value, Self::Error>
        where
            V: serde::de::Visitor<'de>,
        {
            Err(<$error_type>::UnsupportedType("bool"))
        }
    };
    (@primitive $error_type:ty, i8) => {
        fn deserialize_i8<V>(self, _: V) -> Result<V::Value, Self::Error>
        where
            V: serde::de::Visitor<'de>,
        {
            Err(<$error_type>::UnsupportedType("i8"))
        }
    };
    (@primitive $error_type:ty, i16) => {
        fn deserialize_i16<V>(self, _: V) -> Result<V::Value, Self::Error>
        where
            V: serde::de::Visitor<'de>,
        {
            Err(<$error_type>::UnsupportedType("i16"))
        }
    };
    (@primitive $error_type:ty, i32) => {
        fn deserialize_i32<V>(self, _: V) -> Result<V::Value, Self::Error>
        where
            V: serde::de::Visitor<'de>,
        {
            Err(<$error_type>::UnsupportedType("i32"))
        }
    };
    (@primitive $error_type:ty, i64) => {
        fn deserialize_i64<V>(self, _: V) -> Result<V::Value, Self::Error>
        where
            V: serde::de::Visitor<'de>,
        {
            Err(<$error_type>::UnsupportedType("i64"))
        }
    };
    (@primitive $error_type:ty, u8) => {
        fn deserialize_u8<V>(self, _: V) -> Result<V::Value, Self::Error>
        where
            V: serde::de::Visitor<'de>,
        {
            Err(<$error_type>::UnsupportedType("u8"))
        }
    };
    (@primitive $error_type:ty, u16) => {
        fn deserialize_u16<V>(self, _: V) -> Result<V::Value, Self::Error>
        where
            V: serde::de::Visitor<'de>,
        {
            Err(<$error_type>::UnsupportedType("u16"))
        }
    };
    (@primitive $error_type:ty, u32) => {
        fn deserialize_u32<V>(self, _: V) -> Result<V::Value, Self::Error>
        where
            V: serde::de::Visitor<'de>,
        {
            Err(<$error_type>::UnsupportedType("u32"))
        }
    };
    (@primitive $error_type:ty, u64) => {
        fn deserialize_u64<V>(self, _: V) -> Result<V::Value, Self::Error>
        where
            V: serde::de::Visitor<'de>,
        {
            Err(<$error_type>::UnsupportedType("u64"))
        }
    };
    (@primitive $error_type:ty, f32) => {
        fn deserialize_f32<V>(self, _: V) -> Result<V::Value, Self::Error>
        where
            V: serde::de::Visitor<'de>,
        {
            Err(<$error_type>::UnsupportedType("f32"))
        }
    };
    (@primitive $error_type:ty, f64) => {
        fn deserialize_f64<V>(self, _: V) -> Result<V::Value, Self::Error>
        where
            V: serde::de::Visitor<'de>,
        {
            Err(<$error_type>::UnsupportedType("f64"))
        }
    };
    (@primitive $error_type:ty, char) => {
        fn deserialize_char<V>(self, _: V) -> Result<V::Value, Self::Error>
        where
            V: serde::de::Visitor<'de>,
        {
            Err(<$error_type>::UnsupportedType("char"))
        }
    };
    (@primitive $error_type:ty, bytes) => {
        fn deserialize_bytes<V>(self, _: V) -> Result<V::Value, Self::Error>
        where
            V: serde::de::Visitor<'de>,
        {
            Err(<$error_type>::UnsupportedType("bytes"))
        }
    };
    (@primitive $error_type:ty, byte_buf) => {
        fn deserialize_byte_buf<V>(self, _: V) -> Result<V::Value, Self::Error>
        where
            V: serde::de::Visitor<'de>,
        {
            Err(<$error_type>::UnsupportedType("byte_buf"))
        }
    };
    (@primitive $error_type:ty, option) => {
        fn deserialize_option<V>(self, _: V) -> Result<V::Value, Self::Error>
        where
            V: serde::de::Visitor<'de>,
        {
            Err(<$error_type>::UnsupportedType("option"))
        }
    };
    (@primitive $error_type:ty, unit) => {
        fn deserialize_unit<V>(self, _: V) -> Result<V::Value, Self::Error>
        where
            V: serde::de::Visitor<'de>,
        {
            Err(<$error_type>::UnsupportedType("unit"))
        }
    };
    (@primitive $error_type:ty, identifier) => {
        fn deserialize_identifier<V>(self, _: V) -> Result<V::Value, Self::Error>
        where
            V: serde::de::Visitor<'de>,
        {
            Err(<$error_type>::UnsupportedType("identifier"))
        }
    };
    (@primitive $error_type:ty, ignored_any) => {
        fn deserialize_ignored_any<V>(self, _: V) -> Result<V::Value, Self::Error>
        where
            V: serde::de::Visitor<'de>,
        {
            Err(<$error_type>::UnsupportedType("ignored_any"))
        }
    };

    (@compound $error_type:ty, unit_struct) => {
        fn deserialize_unit_struct<V>(
            self,
            _: &'static str,
            _: V,
        ) -> Result<V::Value, Self::Error>
        where
            V: serde::de::Visitor<'de>,
        {
            Err(<$error_type>::UnsupportedType("unit struct"))
        }
    };
    (@compound $error_type:ty, seq) => {
        fn deserialize_seq<V>(self, _: V) -> Result<V::Value, Self::Error>
        where
            V: serde::de::Visitor<'de>,
        {
            Err(<$error_type>::UnsupportedType("sequence"))
        }
    };
    (@compound $error_type:ty, tuple) => {
        fn deserialize_tuple<V>(self, _: usize, _: V) -> Result<V::Value, Self::Error>
        where
            V: serde::de::Visitor<'de>,
        {
            Err(<$error_type>::UnsupportedType("tuple"))
        }
    };
    (@compound $error_type:ty, tuple_struct) => {
        fn deserialize_tuple_struct<V>(
            self,
            _: &'static str,
            _: usize,
            _: V,
        ) -> Result<V::Value, Self::Error>
        where
            V: serde::de::Visitor<'de>,
        {
            Err(<$error_type>::UnsupportedType("tuple struct"))
        }
    };
    (@compound $error_type:ty, map) => {
        fn deserialize_map<V>(self, _: V) -> Result<V::Value, Self::Error>
        where
            V: serde::de::Visitor<'de>,
        {
            Err(<$error_type>::UnsupportedType("map"))
        }
    };
    (@compound $error_type:ty, struct_) => {
        fn deserialize_struct<V>(
            self,
            _: &'static str,
            _: &'static [&'static str],
            _: V,
        ) -> Result<V::Value, Self::Error>
        where
            V: serde::de::Visitor<'de>,
        {
            Err(<$error_type>::UnsupportedType("struct"))
        }
    };
    (@compound $error_type:ty, enum_) => {
        fn deserialize_enum<V>(
            self,
            _: &'static str,
            _: &'static [&'static str],
            _: V,
        ) -> Result<V::Value, Self::Error>
        where
            V: serde::de::Visitor<'de>,
        {
            Err(<$error_type>::UnsupportedType("enum"))
        }
    };
}
