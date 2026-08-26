//! Common helpers for the `multiboot2` and `multiboot2-header` crates.
//!
//! # Features and `no_std` Compatibility
//!
//! This crate is always `no_std`. The `alloc` feature enables heap-allocation
//! helpers. The default `builder` feature enables `alloc` for consistency with
//! the two consuming crates. Disable default features for allocator-free
//! parsing.
//!
//! # Value-add
//!
//! The main value-add of this crate is to abstract away the parsing and
//! construction of Multiboot2 structures. This is more complex than it may
//! sound at first due to the difficulties listed below. It also provides tag
//! iteration.
//!
//! The abstractions provided by this crate serve as the base for the following
//! related structures:
//! - multiboot2:
//!   - boot information
//!   - boot information header (the fixed-size beginning of boot
//!     information)
//!   - boot information tags
//!   - boot information tag header (the fixed-size beginning of a tag)
//! - multiboot2-header:
//!   - Multiboot2 header
//!   - basic header (the fixed-size beginning of a Multiboot2 header)
//!   - header tags
//!   - header tag header (the fixed-size beginning of a tag)
//!
//! # TL;DR: Specific Example
//!
//! To name a specific example, the `multiboot2` crate just needs the following
//! types:
//!
//! - `BootInformationHeader` implementing [`Header`]
//! - `BootInformation` wrapping [`DynSizedStructure`]
//! - `type TagIter<'a> = multiboot2_common::TagIter<'a, TagHeader>`
//!   ([`TagIter`])
//! - `TagHeader` implementing [`Header`]
//! - Structs for each tag, each implementing [`MaybeDynSized`]
//!
//! Then, all the magic using the [`TagIter`] and [`DynSizedStructure::cast`]
//! can easily be utilized.
//!
//! The same correspondingly applies to the structures in `multiboot2-header`.
//!
//! # Design, Solved Problem, and Difficulties along the Way
//!
//! The design choice to have ABI-compatible Rust types in
//! `multiboot2` and `multiboot2-header` mainly influenced the requirements and
//! difficulties. These obstacles, in turn, influenced the design. The outcome
//! is intended to provide a convenient, idiomatic Rust interface.
//!
//! ## Architecture Diagrams
//!
//! The figures in the [README](https://crates.io/crates/multiboot2-common)
//! (currently not embeddable in lib.rs unfortunately) provide an overview of
//! the parsing of Multiboot2 structures and how the definitions from this
//! crate are used.
//!
//! Note that although the diagrams seem complex, most logic is in
//! `multiboot2-common`. For downstream users, the usage is quite simple.
//!
//! ## Multiboot2 Structures
//!
//! Multiboot2 structures are a consecutive chunk of bytes in memory. They use
//! the "header pattern", which means a fixed size and known [`Header`] type
//! indicates the total size of the structure. This is roughly translated to the
//! following Rust base type:
//!
//! ```rust,ignore
//! #[repr(C, align(8))]
//! struct DynStructure {
//!     header: MyHeader,
//!     payload: [u8]
//! }
//! ```
//!
//! Note that these structures can also be nested. So for example, the
//! Multiboot2 boot information contains Multiboot2 tags, and the Multiboot2
//! header contains Multiboot2 header tags - both are themselves **dynamically
//! sized** structures. Their sizes and numbers of elements are known only at
//! runtime.
//!
//! A final `[u8]` field in the structs is the most direct Rust representation.
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
//! Note that some Multiboot2 structures (tags) look like this:
//!
//! ```rust,ignore
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
//! ```rust,ignore
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
//! and `multiboot2-header` crates use a **zero-copy** design by parsing the
//! corresponding raw bytes as **ABI-compatible types** that represent all of
//! their memory.
//!
//! Further, by having ABI-compatible types that fully represent the reality, we
//! can use the same type for parsing **and** for construction, as modelled in
//! the following simplified example:
//!
//! ```rust,ignore
//! /// ABI-compatible tag for parsing.
//! #[repr(C)]
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
//! Hence, the structures can also be built at runtime through the same types
//! used for parsing.
//!
//! ## Creating Fat Pointers with [`ptr_meta`]
//!
//! Fat pointers are a language feature and the base for references to
//! dynamically sized types, such as `&str`, `&[T]`, `dyn T` or
//! `&DynamicallySizedStruct`.
//!
//! Currently, they can't be created using the standard library, but
//! [`ptr_meta`] can be utilized.
//!
//! To create fat pointers with [`ptr_meta`], each tag needs a `Metadata` type
//! which is either `usize` (for DSTs) or `()`. A trait is needed to abstract
//! over sized and unsized types. This is done by [`MaybeDynSized`].
//!
//! ## Multiboot2 Requirements
//!
//! All tags must be 8-byte aligned. The actual payload of tags may be followed
//! by padding zeroes to fill the gap until the next alignment boundary, if
//! necessary. These zeroes are not reflected in the tag's size, but for Rust,
//! must be reflected in the type's memory allocation.
//!
//! ## Rustc Requirements
//!
//! The required allocation space that Rust uses for types is a multiple of the
//! alignment. This means that if we cast between byte slices and specific
//! types, Rust doesn't just see the "trimmed down actual payload" defined by
//! struct members, but also any necessary hidden padding bytes. If we do not
//! account for that padding, for example by casting bytes from a `&[u8; 15]`
//! to an 8-byte-aligned struct, Miri will report an error because Rust expects
//! the allocation to cover 16 bytes.
//!
//! See <https://doc.rust-lang.org/reference/type-layout.html> for information.
//!
//! Further, this means that we can't cast references to smaller structs to
//! larger ones. Once we construct a `Box` using the `new_boxed` helper, we
//! must also ensure that the default
//! [`Layout`] for the underlying type equals the one we manually used for the
//! allocation.
//!
//! ## Parsing and Casting
//!
//! The general idea of parsing is that the lifetime of the original byte slice
//! propagates through to references of target types.
//!
//! First, we need byte slices which are guaranteed to be aligned and are a
//! multiple of the alignment. We have [`BytesRef`] for that. With that, we can
//! create a [`DynSizedStructure`]. This type covers exactly the bytes reported
//! by its header. With the help of [`MaybeDynSized`], we can call
//! [`DynSizedStructure::cast`] to cast this to arbitrary sized or unsized
//! struct types fulfilling the corresponding requirements.
//!
//! This way, one can create Rust structs modeling the structure of the
//! tags, and we only need a single "complicated" type, namely
//! [`DynSizedStructure`].
//!
//! ## Iterating Tags
//!
//! To iterate over the tags of a structure, use [`TagIter`].
//!
//! # Memory Guarantees and Safety Promises
//!
//! The parsing and construction APIs preserve the alignment and padding
//! guarantees discussed above. Parsing APIs report malformed input with
//! appropriate error types. Construction APIs establish the same invariants
//! and may panic when their documented preconditions are violated. Neither
//! malformed input nor a failed invariant may cause undefined behavior.
//!
//! # Stability
//!
//! This crate primarily supports `multiboot2` and `multiboot2-header`. Its
//! public API may evolve with their internals and is not intended as an
//! independent stable abstraction.
//!
//! [`Layout`]: core::alloc::Layout

#![no_std]
// --- BEGIN STYLE CHECKS ---
#![deny(
    clippy::all,
    clippy::cargo,
    clippy::nursery,
    clippy::must_use_candidate,
    clippy::undocumented_unsafe_blocks,
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

#[doc(hidden)]
pub mod test_utils;

#[cfg(feature = "alloc")]
mod boxed;
mod bytes_ref;
mod iter;
mod raw;
mod tag;

#[cfg(feature = "alloc")]
pub use boxed::{clone_dyn, new_boxed};
pub use bytes_ref::BytesRef;
pub use iter::TagIter;
pub use tag::{MaybeDynSized, Tag};

use core::fmt::Debug;
use core::ptr::NonNull;
use core::slice;
use thiserror::Error;

/// The alignment of all Multiboot2 data structures.
pub const ALIGNMENT: usize = 8;

/// A sized header type for [`DynSizedStructure`].
///
/// Note that `header` refers to the header pattern. Thus, depending on the use
/// case, this is not just a tag header. Instead, it refers to all bytes that
/// are fixed and not part of any optional terminating dynamic `[u8]` slice in a
/// [`DynSizedStructure`].
///
/// The alignment of implementors **must** be compatible with the requirements
/// for the corresponding structure, which typically is [`ALIGNMENT`].
///
/// # Safety
///
/// Implementors must be `#[repr(C)]`, have no padding bytes or interior
/// mutability, allow every bit pattern, and have an alignment of at most
/// [`ALIGNMENT`]. Headers are referenced from raw memory and copied byte-wise.
pub unsafe trait Header: Clone + Sized + PartialEq + Eq + Debug {
    /// Returns the total size of the structure in bytes, including the fixed
    /// header and any dynamic payload.
    #[must_use]
    fn total_size(&self) -> usize;

    /// Returns the length of the payload, i.e., the bytes that are additional
    /// to the header. The value is measured in bytes.
    #[must_use]
    fn payload_len(&self) -> usize {
        let total_size = self.total_size();
        assert!(total_size >= size_of::<Self>());
        total_size - size_of::<Self>()
    }

    /// Updates the header with the given `total_size`.
    fn set_size(&mut self, total_size: usize);
}

/// A C ABI-compatible dynamically sized type with a common sized [`Header`]
/// and a dynamic amount of bytes without hidden implicit padding.
///
/// This structure combines a [`Header`] with the data described by that header
/// according to [`Header::total_size`]. Instances guarantee that the memory
/// requirements promised in the crate description are respected.
///
/// This can be a Multiboot2 header tag, information tag, boot information, or
/// a Multiboot2 header. It is the base for **same-size casts** to these
/// corresponding structures using [`DynSizedStructure::cast`]. Depending on the
/// context, the [`Header`] is different (basic header, boot information header,
/// header tag header, or boot information tag header).
///
/// # ABI
/// This type uses the C ABI. The fixed [`Header`] portion is always there.
/// Further, there is a variable amount of payload bytes. Thus, this type can
/// only exist on the heap or references to it can be made by cast via fat
/// pointers. The main constructor is [`DynSizedStructure::ref_from_bytes`].
///
/// As terminating padding might be necessary for the proper Rust type layout,
/// `size_of_val(&self)` might report additional padding bytes that are not
/// reflected by the actual payload. These additional padding bytes however
/// will be reflected in corresponding [`BytesRef`] instances from that this
/// structure was created.
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
    pub fn ref_from_bytes(bytes: BytesRef<'_, H>) -> Result<&Self, MemoryError> {
        let ptr = bytes.as_ptr().cast::<H>();
        // SAFETY: `BytesRef` guarantees alignment and that the buffer covers
        // at least the fixed header size.
        let hdr = unsafe { &*ptr };

        let total_size = hdr.total_size();
        let header_size = size_of::<H>();
        if total_size < header_size {
            return Err(MemoryError::SizeInsufficient(total_size, header_size));
        }
        if total_size > bytes.len() {
            return Err(MemoryError::InvalidReportedTotalSize(
                total_size,
                bytes.len(),
            ));
        }
        let payload_len = total_size - header_size;

        // At this point we know that the memory slice fulfills the base
        // assumptions and requirements. We can now safely create the fat
        // pointer.

        let dst_size = payload_len;
        // Create fat pointer for the DST.
        let ptr = ptr_meta::from_raw_parts(ptr.cast(), dst_size);
        // SAFETY: The allocation was sized from the validated reported total
        // size, so the fat pointer refers to initialized memory.
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
    /// The caller must ensure that `ptr` is readable for at least the size of
    /// [`Header`], and, once its reported total size is known, for that whole
    /// range.
    pub unsafe fn ref_from_ptr<'a>(ptr: NonNull<H>) -> Result<&'a Self, MemoryError> {
        let ptr = ptr.as_ptr().cast_const();

        // Alignment check. All headers are `align(8)`.
        if ptr.cast::<u8>().align_offset(ALIGNMENT) != 0 {
            return Err(MemoryError::WrongAlignment);
        }

        // SAFETY: `ptr` is non-null (from `NonNull`) and now known to be
        // aligned; we only read the reported total size and immediately
        // re-slice that range.
        let hdr = unsafe { &*ptr };
        let total_size = hdr.total_size();
        let header_size = size_of::<H>();
        if total_size < header_size {
            return Err(MemoryError::SizeInsufficient(total_size, header_size));
        }

        // SAFETY: `total_size` came from the validated header and matches the
        // readable byte range for the structure.
        let slice = unsafe { slice::from_raw_parts(ptr.cast::<u8>(), total_size) };
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
    /// The provided `T` may be sized or dynamically sized. The source and
    /// target have the same actual payload size and [`size_of_val`].
    ///
    /// # Panics
    /// Panics if `T` cannot represent the same allocation size. This should not
    /// happen when all types follow their documented requirements.
    pub fn cast<T: MaybeDynSized<Header = H> + ?Sized>(&self) -> &T
    where
        T::Metadata: Default,
    {
        // Thin or fat pointer, depending on type.
        // However, only thin ptr is needed.
        let base_ptr = &raw const *self;

        // This should be a compile-time assertion. However, this is the best
        // location to place it for now.
        assert!(T::BASE_SIZE >= size_of::<H>());

        // Check the size of the allocation is big enough.
        assert!(
            size_of_val(self) >= T::BASE_SIZE,
            "source is too small to be cast to the target type"
        );

        let t_dst_size = T::dst_len(self.header());
        // Creates thin or fat pointer, depending on type.
        let t_ptr = ptr_meta::from_raw_parts(base_ptr.cast(), t_dst_size);
        // SAFETY: `self` is a valid reference and the cast keeps the same
        // allocation; `T::dst_len` determines the matching tail length. The
        // assertion above guarantees the retagged extent stays in bounds.
        let t_ref = unsafe { &*t_ptr };

        assert_eq!(size_of_val(self), size_of_val(t_ref));

        t_ref
    }
}

/// Validates a sequence of padded Multiboot2 (header) tags.
///
/// Both Multiboot2 information tags and Multiboot2 header tags use an 8-byte
/// tag header with the reported tag size stored in bytes 4..8. The reported
/// size excludes alignment padding, but each following tag starts at the next
/// 8-byte boundary.
///
/// Returns `Ok(true)` when a valid end tag is present exactly at the end of the
/// provided byte range, and `Ok(false)` when the byte range ends without an end
/// tag.
pub fn validate_tag_sequence(
    bytes: &[u8],
    mut is_end_tag: impl FnMut(&[u8]) -> bool,
) -> Result<bool, MemoryError> {
    // Common header property for Multiboot2 and Multiboot2 header tags:
    // The `size` property is always at offset 4..8 (the second u32).
    const TAG_HEADER_SIZE: usize = size_of::<u32>() * 2;

    if bytes.as_ptr().align_offset(ALIGNMENT) != 0 {
        return Err(MemoryError::WrongAlignment);
    }

    let mut offset = 0;
    while offset < bytes.len() {
        let remaining = bytes.len() - offset;
        if remaining < TAG_HEADER_SIZE {
            return Err(MemoryError::ShorterThanHeader);
        }

        let tag = &bytes[offset..];
        let total_size =
            u32::from_le_bytes(tag[4..8].try_into().expect("slice has exactly 4 bytes")) as usize;

        if total_size < TAG_HEADER_SIZE {
            return Err(MemoryError::SizeInsufficient(total_size, TAG_HEADER_SIZE));
        }

        let padded_size = total_size
            .checked_add(ALIGNMENT - 1)
            .map(|size| size & !(ALIGNMENT - 1))
            .ok_or(MemoryError::InvalidReportedTotalSize(total_size, remaining))?;
        if padded_size > remaining {
            return Err(MemoryError::InvalidReportedTotalSize(
                padded_size,
                remaining,
            ));
        }

        offset += padded_size;
        if is_end_tag(&tag[..total_size]) {
            if offset == bytes.len() {
                return Ok(true);
            }
            return Err(MemoryError::InvalidReportedTotalSize(offset, bytes.len()));
        }
    }

    Ok(false)
}

/// Errors that may occur when working with memory.
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Error)]
pub enum MemoryError {
    /// The memory points to null.
    #[error("memory points to null")]
    Null,
    /// The memory must be at least [`ALIGNMENT`]-aligned.
    #[error("memory is not properly aligned")]
    WrongAlignment,
    /// The memory must cover at least the length of the sized structure header
    /// type.
    #[error("memory range is shorter than the size of the header structure")]
    ShorterThanHeader,
    /// The size is insufficient to contain at least a valid minimal structure.
    #[error("memory range is shorter than the size of the header structure")]
    SizeInsufficient(usize /* actual */, usize /* expected */),
    /// The buffer misses the terminating padding to the next alignment
    /// boundary. The padding is relevant to satisfy Rustc/Miri, but also the
    /// spec mandates that the padding is added.
    #[error("memory is missing required padding")]
    MissingPadding,
    /// The size-property has an illegal value that can't be fulfilled with the
    /// given bytes.
    #[error(
        "header reports an invalid total size of 0x{0:x} while only 0x{1:x} bytes are available"
    )]
    InvalidReportedTotalSize(usize /* actual */, usize /* expected */),
}

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

        // SAFETY: The tag is repr(C) with the header as first field, any
        // bit pattern is valid, and `BASE_SIZE` matches the ABI.
        unsafe impl MaybeDynSized for CustomSizedTag {
            type Header = DummyTestHeader;

            const BASE_SIZE: usize = size_of::<Self>();

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

        assert_eq!(size_of_val(custom_tag), 16);
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

    #[test]
    fn test_ref_from_ptr_rejects_misaligned() {
        // A misaligned pointer must be reported as an error, not dereferenced
        // (which would be UB, caught by Miri).
        let bytes = AlignedBytes([0x37, 0x13, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        // Guaranteed misaligned: offset 4 into an 8-byte-aligned buffer.
        let misaligned = (&raw const bytes.0[4]).cast::<DummyTestHeader>();
        let ptr = NonNull::new(misaligned.cast_mut()).unwrap();
        // SAFETY: `ptr` is non-null and the constructor will reject the misalignment..
        let result = unsafe { DynSizedStructure::<DummyTestHeader>::ref_from_ptr(ptr) };
        assert_eq!(result, Err(MemoryError::WrongAlignment));
    }

    #[test]
    #[should_panic(expected = "source is too small to be cast to the target type")]
    fn test_cast_rejects_too_small_source() {
        // A sized target larger than the (validly terminated but truncated)
        // source must be rejected before the reference is created, rather
        // than retagging out of bounds (which would be UB under Miri).
        #[repr(C, align(8))]
        struct CustomSizedTag {
            tag_header: DummyTestHeader,
            a: u32,
            b: u32,
        }

        // SAFETY: The tag is repr(C) with the header as first field, any
        // bit pattern is valid, and `BASE_SIZE` matches the ABI.
        unsafe impl MaybeDynSized for CustomSizedTag {
            type Header = DummyTestHeader;

            const BASE_SIZE: usize = size_of::<Self>();

            fn dst_len(_header: &DummyTestHeader) -> Self::Metadata {}
        }

        // Reports a total size of only 8 bytes, i.e., just the header.
        let bytes = AlignedBytes([0x37, 0x13, 0, 0, 8, 0, 0, 0]);
        let tag = DynSizedStructure::ref_from_slice(bytes.borrow()).unwrap();
        // `CustomSizedTag` needs 16 bytes; casting must panic, not read OOB.
        let _ = tag.cast::<CustomSizedTag>();
    }

    #[test]
    fn test_ref_from_slice_rejects_oversized_header() {
        #[rustfmt::skip]
        let bytes = AlignedBytes::new(
            [
                0x37, 0x13, 0, 0,
                /* Tag size */
                24, 0, 0, 0,
                /* Only 8 bytes payload plus padding are available. */
                0, 1, 2, 3,
                4, 5, 6, 7,
            ],
        );

        assert_eq!(
            DynSizedStructure::<DummyTestHeader>::ref_from_slice(bytes.borrow()),
            Err(MemoryError::InvalidReportedTotalSize(24, 16))
        );
    }

    #[test]
    fn test_ref_from_slice_rejects_too_small_reported_size() {
        #[rustfmt::skip]
        let bytes = AlignedBytes::new(
            [
                0x37, 0x13, 0, 0,
                /* Tag size */
                4, 0, 0, 0,
                /* Remaining bytes are irrelevant. */
                0, 1, 2, 3,
                0, 0, 0, 0,
            ],
        );

        assert_eq!(
            DynSizedStructure::<DummyTestHeader>::ref_from_slice(bytes.borrow()),
            Err(MemoryError::SizeInsufficient(4, 8))
        );
    }
}
