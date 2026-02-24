//! Owned serializer trait.

use erased_serde::Serializer as ErasedSerializer;
use serde::ser::Serializer;

/// Bridges owned serializer types with serde's reference-based `Serializer` trait.
pub trait OwnedSerializer: Sized {
    /// Invokes a callback with an erased serializer.
    ///
    /// # Errors
    /// Returns an error if serialization fails.
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
