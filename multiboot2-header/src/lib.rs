//! Library with type definitions and parsing functions for Multiboot2 headers.
//! This library is `no_std` and can be used in bootloaders.
//!
//! # Example
//! ```rust
//! use multiboot2_header::builder::Multiboot2HeaderBuilder;
//! use multiboot2_header::{HeaderTagFlag, HeaderTagISA, InformationRequestHeaderTagBuilder, MbiTagType, RelocatableHeaderTag, RelocatableHeaderTagPreference, Multiboot2Header};
//!
//! // Small example that creates a Multiboot2 header and parses it afterwards.
//!
//! // We create a Multiboot2 header during runtime here. A practical example is that your
//! // program gets the header from a file and parses it afterwards.
//! let mb2_hdr_bytes = Multiboot2HeaderBuilder::new(HeaderTagISA::I386)
//!     .relocatable_tag(RelocatableHeaderTag::new(
//!         HeaderTagFlag::Required,
//!         0x1337,
//!         0xdeadbeef,
//!         4096,
//!         RelocatableHeaderTagPreference::None,
//!     ))
//!     .information_request_tag(
//!         InformationRequestHeaderTagBuilder::new(HeaderTagFlag::Required)
//!             .add_irs(&[MbiTagType::Cmdline, MbiTagType::BootLoaderName]),
//!     )
//!     .build();
//!
//! // Cast bytes in vector to Multiboot2 information structure
//! let mb2_hdr = unsafe { Multiboot2Header::from_addr(mb2_hdr_bytes.as_ptr() as usize) };
//! println!("{:#?}", mb2_hdr);
//!
//! ```

#![no_std]
#![deny(rustdoc::all)]
#![allow(rustdoc::missing_doc_code_examples)]
#![deny(clippy::all)]
#![deny(clippy::missing_const_for_fn)]
#![deny(missing_debug_implementations)]

#[cfg(feature = "builder")]
extern crate alloc;

#[cfg_attr(test, macro_use)]
#[cfg(test)]
extern crate std;

#[cfg_attr(test, macro_use)]
#[cfg(test)]
pub(crate) mod test_utils;

mod address;
mod console;
mod end;
mod entry_efi_32;
mod entry_efi_64;
mod entry_header;
mod framebuffer;
mod header;
mod information_request;
mod module_alignment;
mod relocatable;
mod tags;
mod uefi_bs;

pub use self::address::*;
pub use self::console::*;
pub use self::end::*;
pub use self::entry_efi_32::*;
pub use self::entry_efi_64::*;
pub use self::entry_header::*;
pub use self::framebuffer::*;
pub use self::header::*;
pub use self::information_request::*;
pub use self::module_alignment::*;
pub use self::relocatable::*;
pub use self::tags::*;
pub use self::uefi_bs::*;

use core::mem::size_of;
/// Re-export of [`multiboot2::TagType`] from `multiboot2`-crate as `MbiTagType`, i.e. tags that
/// describe the entries in the Multiboot2 Information Structure (MBI).
pub use multiboot2::TagType as MbiTagType;

/// Trait for all tags that creates a byte array from the tag.
/// Useful in builders to construct a byte vector that
/// represents the Multiboot2 header with all its tags.
#[cfg(feature = "builder")]
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
