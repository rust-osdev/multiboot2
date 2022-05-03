//! Module for the helper trait [`StructAsBytes`].

use crate::{
    AddressHeaderTag, ConsoleHeaderTag, EfiBootServiceHeaderTag, EndHeaderTag,
    EntryAddressHeaderTag, EntryEfi32HeaderTag, EntryEfi64HeaderTag, FramebufferHeaderTag,
    InformationRequestHeaderTag, ModuleAlignHeaderTag, Multiboot2BasicHeader, RelocatableHeaderTag,
};
use core::mem::size_of;

/// Trait for all tags that helps to create a byte array from the tag.
/// Useful in builders to construct a byte vector that
/// represents the Multiboot2 header with all its tags.
pub(crate) trait StructAsBytes: Sized {
    /// Returns the size in bytes of the struct, as known during compile
    /// time. This doesn't use read the "size" field of tags.
    fn byte_size(&self) -> usize {
        size_of::<Self>()
    }

    /// Returns a byte pointer to the begin of the struct.
    fn as_ptr(&self) -> *const u8 {
        self as *const Self as *const u8
    }

    /// Returns the structure as a vector of its bytes.
    /// The length is determined by [`size`].
    fn struct_as_bytes(&self) -> alloc::vec::Vec<u8> {
        let ptr = self.as_ptr();
        let mut vec = alloc::vec::Vec::with_capacity(self.byte_size());
        for i in 0..self.byte_size() {
            vec.push(unsafe { *ptr.add(i) })
        }
        vec
    }
}

impl StructAsBytes for AddressHeaderTag {}
impl StructAsBytes for ConsoleHeaderTag {}
impl StructAsBytes for EndHeaderTag {}
impl StructAsBytes for EntryEfi32HeaderTag {}
impl StructAsBytes for EntryEfi64HeaderTag {}
impl StructAsBytes for EntryAddressHeaderTag {}
impl StructAsBytes for FramebufferHeaderTag {}
impl StructAsBytes for InformationRequestHeaderTag<0> {}
impl StructAsBytes for ModuleAlignHeaderTag {}
impl StructAsBytes for RelocatableHeaderTag {}
impl StructAsBytes for EfiBootServiceHeaderTag {}

impl StructAsBytes for Multiboot2BasicHeader {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_as_bytes() {
        struct Foobar {
            a: u32,
            b: u8,
            c: u128,
        }
        impl StructAsBytes for Foobar {}
        let foo = Foobar {
            a: 11,
            b: 22,
            c: 33,
        };
        let bytes = foo.struct_as_bytes();
        let foo_from_bytes = unsafe { (bytes.as_ptr() as *const Foobar).as_ref().unwrap() };
        assert_eq!(bytes.len(), size_of::<Foobar>());
        assert_eq!(foo.a, foo_from_bytes.a);
        assert_eq!(foo.b, foo_from_bytes.b);
        assert_eq!(foo.c, foo_from_bytes.c);
    }
}
