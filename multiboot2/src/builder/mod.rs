//! Module for the builder-feature.

mod boxed_dst;
mod information;

// This must by public to support external people to create boxed DSTs.
pub use boxed_dst::BoxedDst;
pub use information::InformationBuilder;

/// Helper trait for all structs that need to be serialized that do not
/// implement `TagTrait`.
pub trait AsBytes: Sized {
    fn as_bytes(&self) -> &[u8] {
        let ptr = core::ptr::addr_of!(*self);
        let size = core::mem::size_of::<Self>();
        unsafe { core::slice::from_raw_parts(ptr.cast(), size) }
    }
}
