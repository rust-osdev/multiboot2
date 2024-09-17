//! Common helpers for the `multiboot2` and `multiboot2-header` crates.
//!
//! # Value-add
//!
//! The main value-add of this crate is to abstract away the parsing and
//! construction of Multiboot2 structures. This is more complex as it may sound
//! at first due to the difficulties listed below.
//!
//! The abstractions provided by this crate serve as the base to work with the
//! following structures:
//! - multiboot2:
//!   - boot information structure (whole)
//!   - boot information tags
//! - multiboot2-header:
//!   - header structure (whole)
//!   - header tags
//!
//! # Solved Problem & Difficulties Along the Way
//!
//! Firstly, the design choice to have ABI-compatible rusty types influenced the
//! requirements and difficulties along the way. They, on the other side,
//! influenced the design. The outcome is what we perceive as the optimal rusty
//! and convenient solution.
//!
//! ## Multiboot2 Structures
//!
//! Multiboot2 structures are a consecutive chunk of bytes in memory. They use
//! the "header pattern", which means a fixed size and known [`Header`] type
//! indicates the total size of the structure. This is roughly translated to the
//! following rusty base type:
//!
//! ```ignore
//! #[repr(C, align(8))]
//! struct DynStructure {
//!     header: MyHeader,
//!     payload: [u8]
//! }
//! ```
//!
//! Note that these structures can also be nested. So for example, the
//! Multiboot2 boot information contains Multiboot2 tags, and the Multiboot2
//! header contains Multiboot2 header tags - both are itself **dynamically
//! sized** structures. This means, you can know the size (and amount of
//! elements) **only at runtime!**
//!
//! A final `[u8]` field in the structs is the most rusty way to model this.
//! However, this makes the type a Dynamically Sized Type (DST). To create
//! references to these types from a byte slice, one needs fat pointers. They
//! are a language feature currently not constructable with stable Rust.
//! Luckily, we can utilize [`ptr_meta`].
//!
//! Figure 1 in the [README](https://crates.io/crates/multiboot2-common)
//! (currently not embeddable in lib.rs unfortunately) provides an overview of
//! Multiboot2 structures.
//!
//! ## Dynamic and Sized Structs in Rust
//!
//! Note that we also have structures (tags) in Multiboot2 that looks like this:
//!
//! ```ignore
//! #[repr(C, align(8))]
//! struct DynStructure {
//!     header: MyHeader,
//!     // Not just [`u8`]
//!     payload: [SomeType]
//! }
//! ```
//!
//! or
//!
//! ```ignore
//! #[repr(C, align(8))]
//! struct CommandLineTag {
//!     header: TagHeader,
//!     start: u32,
//!     end: u32,
//!     // More than just the base header before the dynamic portion
//!     data: [u8]
//! }
//! ```
//!
//! ## Chosen Design
//!
//! The overall common abstractions needed to solve the problems mentioned in
//! this section are also mainly influenced by the fact that the `multiboot2`
//! and `multiboot2-header` crates use a **zero-copy** design for parsing
//! the corresponding structures.
//!
//! Further, by having **ABI-compatible types** that fully represent the
//! reality, we can use the same type for parsing **and** for construction,
//! as modelled in the following simplified example:
//!
//! ```rust,ignore
//! /// ABI-compatible tag for parsing.
//! pub struct MemoryMapTag {
//!     header: TagHeader,
//!     entry_size: u32,
//!     entry_version: u32,
//!     areas: [MemoryArea],
//! }
//!
//! impl MemoryMapTag {
//!     // We can also create an ABI-compatible structure of that type.
//!     pub fn new(areas: &[MemoryArea]) -> Box<Self> {
//!         // omitted
//!     }
//! }
//! ```
//!
//! Hence, the structures can also be build at runtime. This is what we
//! consider **idiomatic and rusty**.
//!
//! ## Creating Fat Pointers with [`ptr_meta`]
//!
//! To create fat pointers with [`ptr_meta`], each tag needs a `Metadata` type
//! which is either `usize` (for DSTs) or `()`. A trait is needed to abstract
//! above sized or unsized types.
//!
//! ## Multiboot2 Requirements
//!
//! All tags must be 8-byte aligned. The actual payload of tags may be followed
//! by padding zeroes to fill the gap until the next alignment boundary, if
//! necessary. These zeroes are not reflected in the tag's size, but for Rust,
//! must be reflected in the memory allocation size.
//!
//! ## Rustc Requirements
//!
//! The required allocation space that Rust uses for types is a multiple of the
//! alignment. This means that if we cast between byte slices and specific
//! types, Rust doesn't just see the "trimmed down actual payload" defined by
//! struct members, but also any necessary, but hidden, padding bytes. If we
//! don't guarantee the correct is not the case, for example we cast the bytes
//! from a `&[u8; 15]` to an 8-byte aligned struct, Miri will complain as it
//! expects `&[u8; 16]`.
//!
//! See <https://doc.rust-lang.org/reference/type-layout.html> for information.
//!
//! Further, this also means that we can't cast references to smaller structs
//! to bigger ones. Also, once we construct a `Box` on the heap and construct
//! it using the [`new_boxed`] helper, we must ensure that the default
//! [`Layout`] for the underlying type equals the one we manually used for the
//! allocation.
//!
//! # Architecture & Provided Abstractions
//!
//! Figure 2 in the [README](https://crates.io/crates/multiboot2-common)
//! (currently not embeddable in lib.rs unfortunately) provides an overview of
//! the parsing of Multiboot2 structures and how the definitions from this
//! crate are used.
//!
//! ## Parsing and Casting
//!
//! First, we need byte slices which are guaranteed to be aligned and are a
//! multiple of the alignment. We have [`BytesRef`] for that. With that, we can
//! create a [`DynSizedStructure`]. This is a rusty type that owns all the bytes
//! it owns, according to the size reported by its header. Using this type
//! and with the help of [`MaybeDynSized`], we can call
//! [`DynSizedStructure::cast`] to cast this to arbitrary sized or unsized
//! struct types fulfilling the corresponding requirements.
//!
//! This way, one can create nice rusty structs modeling the structure of the
//! tags, and we only need a single "complicated" type, namely
//! [`DynSizedStructure`].
//!
//! ## Iterating Tags
//!
//! To iterate over the tags of a structure, use [`TagIter`].
//!
//! # Memory Guarantees and Safety Promises
//!
//! For the parsing and construction of Multiboot2 structures, the alignment
//! and necessary padding bytes as discussed above are guaranteed. When types
//! are constructed, they return Results with appropriate error types. If
//! during runtime something goes wrong, for example due to malformed tags,
//! panics guarantee that no UB will happen.
//!
//! # No Public API
//!
//! Not meant as stable public API for others outside Multiboot2.
//!
//! [`Layout`]: core::alloc::Layout

#![no_std]
#![cfg_attr(feature = "unstable", feature(error_in_core))]
// --- BEGIN STYLE CHECKS ---
#![deny(
    clippy::all,
    clippy::cargo,
    clippy::must_use_candidate,
    clippy::nursery,
    missing_debug_implementations,
    missing_docs,
    rustdoc::all
)]
#![allow(clippy::multiple_crate_versions)]
// --- END STYLE CHECKS ---

#[cfg_attr(test, macro_use)]
#[cfg(test)]
extern crate std;

#[cfg(feature = "alloc")]
extern crate alloc;

#[allow(unused)]
pub mod test_utils;

#[cfg(feature = "alloc")]
mod boxed;
mod bytes_ref;
mod iter;
mod tag;

#[cfg(feature = "alloc")]
pub use boxed::{clone_dyn, new_boxed};
pub use bytes_ref::BytesRef;
pub use iter::TagIter;
pub use tag::{MaybeDynSized, Tag};

use core::fmt::Debug;
use core::mem;
use core::ptr;
use core::ptr::NonNull;
use core::slice;

/// The alignment of all Multiboot2 data structures.
pub const ALIGNMENT: usize = 8;

/// A sized header type for [`DynSizedStructure`].
///
/// Note that `header` refers to the header pattern. Thus, depending on the use
/// case, this is not just a tag header. Instead, it refers to all bytes that
/// are fixed and not part of any optional terminating dynamic `[u8]` slice in a
/// [`DynSizedStructure`].
///
/// The alignment of implementors **must** be the compatible with the demands
/// for the corresponding structure, which typically is [`ALIGNMENT`].
pub trait Header: Clone + Sized + PartialEq + Eq + Debug {
    /// Returns the length of the payload, i.e., the bytes that are additional
    /// to the header. The value is measured in bytes.
    #[must_use]
    fn payload_len(&self) -> usize;

    /// Returns the total size of the struct, thus the size of the header itself
    /// plus [`Header::payload_len`].
    #[must_use]
    fn total_size(&self) -> usize {
        mem::size_of::<Self>() + self.payload_len()
    }

    /// Updates the header with the given `total_size`.
    fn set_size(&mut self, total_size: usize);
}

/// An C ABI-compatible dynamically sized type with a common sized [`Header`]
/// and a dynamic amount of bytes.
///
/// This structures owns all its bytes, unlike [`Header`]. Instances guarantees
/// that the memory requirements promised in the crates description are
/// respected.
///
/// This can be a Multiboot2 header tag, information tag, boot information, or
/// a Multiboot2 header. Depending on the context, the [`Header`] is different.
///
/// # ABI
/// This type uses the C ABI. The fixed [`Header`] portion is always there.
/// Further, there is a variable amount of payload bytes. Thus, this type can
/// only exist on the heap or references to it can be made by cast via fat
/// pointers.
///
/// As there might be padding necessary for the proper Rust layout,
/// `size_of_val(&self)` might report additional padding bytes that are not
/// reflected by the actual payload. These additional padding bytes however
/// will be reflected in corresponding [`BytesRef`] instances.
#[derive(Debug, PartialEq, Eq, ptr_meta::Pointee)]
#[repr(C, align(8))]
pub struct DynSizedStructure<H: Header> {
    header: H,
    payload: [u8],
    // Plus optional padding bytes to next alignment boundary, which are not
    // reflected here. However, Rustc allocates them anyway and expects them
    // to be there.
    // See <https://doc.rust-lang.org/reference/type-layout.html>.
}

impl<H: Header> DynSizedStructure<H> {
    /// Creates a new fat-pointer backed reference to a [`DynSizedStructure`]
    /// from the given [`BytesRef`].
    pub fn ref_from_bytes(bytes: BytesRef<H>) -> Result<&Self, MemoryError> {
        let ptr = bytes.as_ptr().cast::<H>();
        let hdr = unsafe { &*ptr };

        if hdr.payload_len() > bytes.len() {
            return Err(MemoryError::InvalidReportedTotalSize);
        }

        // At this point we know that the memory slice fulfills the base
        // assumptions and requirements. Now, we safety can create the fat
        // pointer.

        let dst_size = hdr.payload_len();
        // Create fat pointer for the DST.
        let ptr = ptr_meta::from_raw_parts(ptr.cast(), dst_size);
        let reference = unsafe { &*ptr };
        Ok(reference)
    }

    /// Creates a new fat-pointer backed reference to a [`DynSizedStructure`]
    /// from the given `&[u8]`.
    pub fn ref_from_slice(bytes: &[u8]) -> Result<&Self, MemoryError> {
        let bytes = BytesRef::<H>::try_from(bytes)?;
        Self::ref_from_bytes(bytes)
    }

    /// Creates a new fat-pointer backed reference to a [`DynSizedStructure`]
    /// from the given thin pointer to the [`Header`]. It reads the total size
    /// from the header.
    ///
    /// # Safety
    /// The caller must ensure that the function operates on valid memory.
    pub unsafe fn ref_from_ptr<'a>(ptr: NonNull<H>) -> Result<&'a Self, MemoryError> {
        let ptr = ptr.as_ptr().cast_const();
        let hdr = unsafe { &*ptr };

        let slice = unsafe { slice::from_raw_parts(ptr.cast::<u8>(), hdr.total_size()) };
        Self::ref_from_slice(slice)
    }

    /// Returns the underlying [`Header`].
    pub const fn header(&self) -> &H {
        &self.header
    }

    /// Returns the underlying payload.
    pub const fn payload(&self) -> &[u8] {
        &self.payload
    }

    /// Performs a memory-safe same-size cast from the base-structure to a
    /// specific [`MaybeDynSized`]. The idea here is to cast the generic
    /// mostly semantic-free version to a specific type with fields that have
    /// a clear semantic.
    ///
    /// The provided `T` of type [`MaybeDynSized`] might be may be sized type
    /// or DST. This depends on the type. However, the source and the target
    /// both will have the same actual payload size and the same
    /// [`size_of_val`].
    ///
    /// # Panic
    /// Panics if base assumptions are violated. For example, the
    /// `T` of type [`MaybeDynSized`] must allow proper same-size casting to it.
    ///
    /// # Safety
    /// This function is safe due to various sanity checks and the overall
    /// memory assertions done while constructing this type.
    ///
    /// # Panics
    /// This panics if there is a size mismatch. However, this should never be
    /// the case if all types follow their documented requirements.
    ///
    /// [`size_of_val`]: mem::size_of_val
    pub fn cast<T: MaybeDynSized<Header = H> + ?Sized>(&self) -> &T {
        let base_ptr = ptr::addr_of!(*self);

        // This should be a compile-time assertion. However, this is the best
        // location to place it for now.
        assert!(T::BASE_SIZE >= mem::size_of::<H>());

        let t_dst_size = T::dst_len(self.header());
        let t_ptr = ptr_meta::from_raw_parts(base_ptr.cast(), t_dst_size);
        let t_ref = unsafe { &*t_ptr };

        assert_eq!(mem::size_of_val(self), mem::size_of_val(t_ref));

        t_ref
    }
}

/// Errors that may occur when working with memory.
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, derive_more::Display)]
pub enum MemoryError {
    /// The memory points to null.
    Null,
    /// The memory must be at least [`ALIGNMENT`]-aligned.
    WrongAlignment,
    /// The memory must cover at least the length of the sized structure header
    /// type.
    ShorterThanHeader,
    /// The buffer misses the terminating padding to the next alignment
    /// boundary. The padding is relevant to satisfy Rustc/Miri, but also the
    /// spec mandates that the padding is added.
    MissingPadding,
    /// The size-property has an illegal value that can't be fulfilled with the
    /// given bytes.
    InvalidReportedTotalSize,
}

#[cfg(feature = "unstable")]
impl core::error::Error for MemoryError {}

/// Increases the given size to the next alignment boundary, if it is not a
/// multiple of the alignment yet.
///
/// This is relevant as in Rust's [type layout], the allocated size of a type is
/// always a multiple of the alignment, even if the type is smaller.
///
/// [type layout]: https://doc.rust-lang.org/reference/type-layout.html
#[must_use]
pub const fn increase_to_alignment(size: usize) -> usize {
    let mask = ALIGNMENT - 1;
    (size + mask) & !mask
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{AlignedBytes, DummyTestHeader};
    use core::borrow::Borrow;

    #[test]
    fn test_increase_to_alignment() {
        assert_eq!(increase_to_alignment(0), 0);
        assert_eq!(increase_to_alignment(1), 8);
        assert_eq!(increase_to_alignment(7), 8);
        assert_eq!(increase_to_alignment(8), 8);
        assert_eq!(increase_to_alignment(9), 16);
    }

    #[test]
    fn test_cast_generic_tag_to_sized_tag() {
        #[repr(C)]
        struct CustomSizedTag {
            tag_header: DummyTestHeader,
            a: u32,
            b: u32,
        }

        impl MaybeDynSized for CustomSizedTag {
            type Header = DummyTestHeader;

            const BASE_SIZE: usize = mem::size_of::<Self>();

            fn dst_len(_header: &DummyTestHeader) -> Self::Metadata {}
        }

        let bytes = AlignedBytes([
            /* id: 0xffff_ffff */
            0xff_u8, 0xff_u8, 0xff_u8, 0xff_u8, /* id: 16 */
            16, 0, 0, 0, /* field a: 0xdead_beef */
            0xef, 0xbe, 0xad, 0xde, /* field b: 0x1337_1337 */
            0x37, 0x13, 0x37, 0x13,
        ]);
        let tag = DynSizedStructure::ref_from_slice(bytes.borrow()).unwrap();
        let custom_tag = tag.cast::<CustomSizedTag>();

        assert_eq!(mem::size_of_val(custom_tag), 16);
        assert_eq!(custom_tag.a, 0xdead_beef);
        assert_eq!(custom_tag.b, 0x1337_1337);
    }

    #[test]
    fn test_cast_generic_tag_to_self() {
        #[rustfmt::skip]
        let bytes = AlignedBytes::new(
            [
                0x37, 0x13, 0, 0,
                /* Tag size */
                18, 0, 0, 0,
                /* Some payload.  */
                0, 1, 2, 3,
                4, 5, 6, 7,
                8, 9,
                // Padding
                0, 0, 0, 0, 0, 0
            ],
        );
        let tag = DynSizedStructure::ref_from_slice(bytes.borrow()).unwrap();

        // Main objective here is also that this test passes Miri.
        let tag = tag.cast::<DynSizedStructure<DummyTestHeader>>();
        assert_eq!(tag.header().typ(), 0x1337);
        assert_eq!(tag.header().size(), 18);
    }
}
