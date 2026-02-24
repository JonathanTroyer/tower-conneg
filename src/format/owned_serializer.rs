//! Owned serializer trait for bridging serde's reference-based serializers.
//!
//! # Why This Exists
//!
//! Serde's [`Serializer`] trait is typically implemented on references (`&mut T`), not
//! owned types. For example, `serde_json::Serializer` implements `Serializer` via
//! `&mut serde_json::Serializer`. This is efficient because it avoids moving the
//! serializer on each operation.
//!
//! However, our [`Format::serializer()`](super::Format::serializer) method needs to
//! return an owned value that the caller can store and use. We can't return a reference
//! because there's nothing for it to reference yet.
//!
//! # The Solution
//!
//! [`OwnedSerializer`] bridges this gap:
//!
//! 1. `Format::serializer()` returns an owned type implementing `OwnedSerializer`
//! 2. The caller stores this owned value
//! 3. When ready to serialize, call `as_serializer()` to get the reference that
//!    implements serde's `Serializer`
//!
//! # How It Works
//!
//! The trait uses a higher-ranked trait bound (`for<'a>`) to express that the
//! serializer can be borrowed for any lifetime. The private helper trait
//! `OwnedSerializerLifetime` captures the relationship between the owned type and
//! the borrowed serializer type at each specific lifetime.
//!
//! Most serde serializers (like `serde_json::Serializer<W>`) automatically implement
//! `OwnedSerializer` through the blanket impl, because `&mut T: Serializer` holds.

use serde::Serializer;

mod private {
    use serde::Serializer;

    pub trait OwnedSerializerLifetime<'a, Extra = &'a Self> {
        type SerializerLt: Serializer;

        fn get_serializer(&'a mut self) -> Self::SerializerLt;
    }

    impl<'a, T> OwnedSerializerLifetime<'a> for T
    where
        T: ?Sized,
        &'a mut T: Serializer,
    {
        type SerializerLt = &'a mut T;

        fn get_serializer(&'a mut self) -> Self::SerializerLt {
            self
        }
    }
}

/// A type that owns a serializer and can provide a reference to it.
///
/// This trait bridges owned serializer types with serde's reference-based
/// [`Serializer`] trait, allowing formats to manage serializer lifetimes.
pub trait OwnedSerializer: for<'a> private::OwnedSerializerLifetime<'a> {
    /// The serializer type returned by [`as_serializer`](Self::as_serializer).
    type Serializer<'a>: Serializer
    where
        Self: 'a;

    /// Returns a reference to the underlying serializer.
    fn as_serializer(&mut self) -> Self::Serializer<'_>;
}

impl<T> OwnedSerializer for T
where
    for<'a> T: private::OwnedSerializerLifetime<'a>,
{
    type Serializer<'a>
        = <Self as private::OwnedSerializerLifetime<'a>>::SerializerLt
    where
        Self: 'a;

    fn as_serializer(&mut self) -> Self::Serializer<'_> {
        <Self as private::OwnedSerializerLifetime<'_>>::get_serializer(self)
    }
}
