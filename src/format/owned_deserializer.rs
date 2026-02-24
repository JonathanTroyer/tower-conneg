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
//! 2. The caller stores this owned value
//! 3. When ready to deserialize, call `as_deserializer()` to get the reference that
//!    implements serde's `Deserializer`
//!
//! # How It Works
//!
//! The trait uses a higher-ranked trait bound (`for<'a>`) to express that the
//! deserializer can be borrowed for any lifetime. The private helper trait
//! `OwnedDeserializerLifetime` captures the relationship between the owned type,
//! the input data lifetime (`'de`), and the borrowed deserializer type.
//!
//! Most serde deserializers (like `serde_json::Deserializer<R>`) automatically implement
//! `OwnedDeserializer` through the blanket impl, because `&mut T: Deserializer<'de>` holds.

use serde::Deserializer;

mod private {
    use serde::Deserializer;

    pub trait OwnedDeserializerLifetime<'de, 'a, Extra = &'a Self> {
        type DeserializerLt: Deserializer<'de>;

        fn get_deserializer(&'a mut self) -> Self::DeserializerLt;
    }

    impl<'de, 'a, T> OwnedDeserializerLifetime<'de, 'a> for T
    where
        T: ?Sized,
        &'a mut T: Deserializer<'de>,
    {
        type DeserializerLt = &'a mut T;

        fn get_deserializer(&'a mut self) -> Self::DeserializerLt {
            self
        }
    }
}

/// A type that owns a deserializer and can provide a reference to it.
///
/// This trait bridges owned deserializer types with serde's reference-based
/// [`Deserializer`] trait, allowing formats to manage deserializer lifetimes.
pub trait OwnedDeserializer<'de>: for<'a> private::OwnedDeserializerLifetime<'de, 'a> {
    /// The deserializer type returned by [`as_deserializer`](Self::as_deserializer).
    type Deserializer<'a>: Deserializer<'de>
    where
        Self: 'a;

    /// Returns a reference to the underlying deserializer.
    fn as_deserializer(&mut self) -> Self::Deserializer<'_>;
}

impl<'de, T> OwnedDeserializer<'de> for T
where
    for<'a> T: private::OwnedDeserializerLifetime<'de, 'a>,
{
    type Deserializer<'a>
        = <Self as private::OwnedDeserializerLifetime<'de, 'a>>::DeserializerLt
    where
        Self: 'a;

    fn as_deserializer(&mut self) -> Self::Deserializer<'_> {
        <Self as private::OwnedDeserializerLifetime<'de, '_>>::get_deserializer(self)
    }
}
