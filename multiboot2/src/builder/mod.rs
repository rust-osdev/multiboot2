//! Module for the builder-feature.

mod boxed_dst;
mod information;

// This must be public to support external people to create boxed DSTs.
pub use boxed_dst::BoxedDst;
pub use information::InformationBuilder;

/// Helper trait for all structs that need to be serialized that do not
/// implement [`TagTrait`].
///
/// [`TagTrait`]: crate::TagTrait
pub trait AsBytes: Sized {
    /// Returns the raw bytes of the type.
    fn as_bytes(&self) -> &[u8] {
        let ptr = core::ptr::addr_of!(*self);
        let size = core::mem::size_of::<Self>();
        unsafe { core::slice::from_raw_parts(ptr.cast(), size) }
    }
}
