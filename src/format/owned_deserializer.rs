//! Owned deserializer trait for bridging serde's reference-based deserializers.
//!
//! # Why This Exists
//!
//! Serde's [`Deserializer`] trait is typically implemented on references (`&mut T`), not
//! owned types. For example, `serde_json::Deserializer` implements `Deserializer` via
//! `&mut serde_json::Deserializer`. This is efficient because it avoids moving the
//! deserializer on each operation.
//!
//! However, our [`Format::deserializer()`](super::Format::deserializer) method needs to
//! return an owned value that the caller can store and use. We can't return a reference
//! because there's nothing for it to reference yet.
//!
//! # The Solution
//!
//! [`OwnedDeserializer`] bridges this gap:
//!
//! 1. `Format::deserializer()` returns an owned type implementing `OwnedDeserializer`
//! 2. Call `into_deserializer()` to consume it and get a `Deserializer`
//!
//! # Two Patterns
//!
//! Deserializers come in two flavors:
//!
//! 1. **Borrowable** (like `serde_json::Deserializer`): Implements `Deserializer` for
//!    `&mut Self`. Wrap these in [`Borrowable`] which implements `Deserializer` by
//!    delegating to the inner `&mut T`.
//!
//! 2. **Consumable** (like `serde_urlencoded::Deserializer`): Implements `Deserializer`
//!    only for the owned type. Wrap these in [`Consumable`] which simply returns the
//!    inner deserializer.

use serde::de::Visitor;
use serde::{Deserializer, forward_to_deserialize_any};

/// A type that owns a deserializer and can be consumed to yield it.
///
/// This trait bridges owned deserializer types with serde's [`Deserializer`] trait,
/// allowing formats to manage deserializer lifetimes.
///
/// Use [`Borrowable`] for deserializers where `&mut T: Deserializer`, and
/// [`Consumable`] for deserializers where `T: Deserializer` directly.
pub trait OwnedDeserializer<'de>: Sized {
    /// The deserializer type returned by [`into_deserializer`](Self::into_deserializer).
    type Deserializer: Deserializer<'de>;

    /// Consumes this wrapper and returns the underlying deserializer.
    fn into_deserializer(self) -> Self::Deserializer;
}

/// Wrapper for deserializers where `&mut T: Deserializer` (borrowable pattern).
///
/// This wrapper implements [`Deserializer`] by delegating to `&mut self.0`,
/// allowing borrowable deserializers to be used with [`OwnedDeserializer`].
///
/// # Example
///
/// ```ignore
/// use tower_conneg::Borrowable;
///
/// fn deserializer<'a>(bytes: &'a [u8]) -> Borrowable<serde_json::Deserializer<...>> {
///     Borrowable(serde_json::Deserializer::from_slice(bytes))
/// }
/// ```
#[derive(Debug)]
pub struct Borrowable<T>(pub T);

impl<'de, T> Deserializer<'de> for Borrowable<T>
where
    for<'a> &'a mut T: Deserializer<'de>,
{
    type Error = erased_serde::Error;

    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        (&mut self.0)
            .deserialize_any(visitor)
            .map_err(serde::de::Error::custom)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
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

/// Wrapper for deserializers that consume `self` (consumable pattern).
///
/// This wrapper simply returns the inner deserializer when consumed.
///
/// # Example
///
/// ```ignore
/// use tower_conneg::Consumable;
///
/// fn deserializer<'a>(bytes: &'a [u8]) -> Consumable<SomeDeserializer<'a>> {
///     Consumable::new(SomeDeserializer::new(bytes))
/// }
/// ```
#[derive(Debug)]
pub struct Consumable<D>(D);

impl<D> Consumable<D> {
    /// Creates a new wrapper around a consumable deserializer.
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
