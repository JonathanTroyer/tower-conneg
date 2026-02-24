//! Owned deserializer trait.

use serde::Deserializer;
use serde::de::Visitor;

/// Bridges owned deserializer types with serde's `Deserializer` trait.
///
/// Use [`Borrowable`] for `&mut T: Deserializer`, [`Consumable`] for `T: Deserializer`.
pub trait OwnedDeserializer<'de>: Sized {
    /// The deserializer type.
    type Deserializer: Deserializer<'de>;

    /// Returns the underlying deserializer.
    fn into_deserializer(self) -> Self::Deserializer;
}

/// Wrapper for deserializers where `&mut T: Deserializer`.
#[derive(Debug)]
pub struct Borrowable<T>(pub T);

macro_rules! forward_deserialize_methods {
    // Simple methods: just take a visitor
    (simple: $($method:ident),* $(,)?) => {
        $(
            fn $method<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
            where
                V: Visitor<'de>,
            {
                (&mut self.0).$method(visitor).map_err(serde::de::Error::custom)
            }
        )*
    };

    // Methods with a single &'static str parameter (name)
    (name: $($method:ident),* $(,)?) => {
        $(
            fn $method<V>(
                mut self,
                name: &'static str,
                visitor: V,
            ) -> Result<V::Value, Self::Error>
            where
                V: Visitor<'de>,
            {
                (&mut self.0).$method(name, visitor).map_err(serde::de::Error::custom)
            }
        )*
    };

    // Methods with len: usize parameter
    (len: $($method:ident),* $(,)?) => {
        $(
            fn $method<V>(
                mut self,
                len: usize,
                visitor: V,
            ) -> Result<V::Value, Self::Error>
            where
                V: Visitor<'de>,
            {
                (&mut self.0).$method(len, visitor).map_err(serde::de::Error::custom)
            }
        )*
    };
}

impl<'de, T> serde::Deserializer<'de> for Borrowable<T>
where
    for<'a> &'a mut T: Deserializer<'de>,
{
    type Error = erased_serde::Error;

    forward_deserialize_methods!(simple:
        deserialize_any,
        deserialize_bool,
        deserialize_i8,
        deserialize_i16,
        deserialize_i32,
        deserialize_i64,
        deserialize_i128,
        deserialize_u8,
        deserialize_u16,
        deserialize_u32,
        deserialize_u64,
        deserialize_u128,
        deserialize_f32,
        deserialize_f64,
        deserialize_char,
        deserialize_str,
        deserialize_string,
        deserialize_bytes,
        deserialize_byte_buf,
        deserialize_option,
        deserialize_unit,
        deserialize_seq,
        deserialize_map,
        deserialize_identifier,
        deserialize_ignored_any,
    );

    forward_deserialize_methods!(name:
        deserialize_unit_struct,
        deserialize_newtype_struct,
    );

    forward_deserialize_methods!(len:
        deserialize_tuple,
    );

    fn deserialize_tuple_struct<V>(
        mut self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        (&mut self.0)
            .deserialize_tuple_struct(name, len, visitor)
            .map_err(serde::de::Error::custom)
    }

    fn deserialize_struct<V>(
        mut self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        (&mut self.0)
            .deserialize_struct(name, fields, visitor)
            .map_err(serde::de::Error::custom)
    }

    fn deserialize_enum<V>(
        mut self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        (&mut self.0)
            .deserialize_enum(name, variants, visitor)
            .map_err(serde::de::Error::custom)
    }
}

impl<'de, T> OwnedDeserializer<'de> for Borrowable<T>
where
    for<'a> &'a mut T: Deserializer<'de>,
{
    type Deserializer = Self;

    fn into_deserializer(self) -> Self {
        self
    }
}

/// Wrapper for deserializers that consume `self`.
#[derive(Debug)]
pub struct Consumable<D>(D);

impl<D> Consumable<D> {
    /// Wraps a consumable deserializer.
    pub fn new(deserializer: D) -> Self {
        Self(deserializer)
    }
}

impl<'de, D> OwnedDeserializer<'de> for Consumable<D>
where
    D: Deserializer<'de>,
{
    type Deserializer = D;

    fn into_deserializer(self) -> D {
        self.0
    }
}
