//! Owned serializer trait for bridging serde's reference-based serializers.
//!
//! Serde's [`Serializer`](serde::Serializer) trait is typically implemented on references
//! (`&mut T`), not owned types. This module provides [`OwnedSerializer`] which abstracts
//! over this pattern, allowing formats to return owned serializer values.

use erased_serde::Serializer as ErasedSerializer;
use serde::ser::Serializer;

/// A type that owns a serializer and can provide erased access to it.
///
/// This trait bridges owned serializer types with serde's reference-based [`Serializer`] trait.
/// The blanket implementation handles the common case where `&mut T: Serializer`.
pub trait OwnedSerializer: Sized {
    /// Invokes a callback with an erased serializer.
    ///
    /// # Errors
    ///
    /// Returns an error if the callback fails.
    fn with_erased(
        self,
        f: &mut dyn FnMut(&mut dyn ErasedSerializer) -> erased_serde::Result<()>,
    ) -> erased_serde::Result<()>;
}

impl<S> OwnedSerializer for S
where
    for<'a> &'a mut S: Serializer,
{
    fn with_erased(
        mut self,
        f: &mut dyn FnMut(&mut dyn ErasedSerializer) -> erased_serde::Result<()>,
    ) -> erased_serde::Result<()> {
        let mut erased = <dyn ErasedSerializer>::erase(&mut self);
        f(&mut erased)
    }
}
