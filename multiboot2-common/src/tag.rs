//! Module for the traits [`MaybeDynSized`] and [`Tag`].

use crate::{DynSizedStructure, Header};
use core::slice;
use ptr_meta::Pointee;

/// A trait to abstract sized and unsized structures (DSTs). It enables
/// casting a [`DynSizedStructure`] to sized or unsized structures using
/// [`DynSizedStructure::cast`].
///
/// Structs that are a DST must provide a **correct** [`MaybeDynSized::dst_len`]
/// implementation. The needed metadata type is either `()` for sized types or
/// `usize` for dynamically sized types. For sized types, there is a default
/// implementation. Only dynamically sized types need to implement
/// [`MaybeDynSized::dst_len`].
///
/// # Safety
///
/// Implementors must be `#[repr(C)]`, start with `Self::Header`, have an
/// alignment of at most [`ALIGNMENT`], and allow every bit pattern.
///
/// [`MaybeDynSized::BASE_SIZE`], [`MaybeDynSized::dst_len`], and
/// [`Header::total_size`] must correctly describe the initialized,
/// contiguous memory backing the value. Incorrect sizes or implicit padding
/// within the reported range can cause out-of-bounds references. Trailing
/// padding beyond that range is fine.
///
/// [`ID`]: Tag::ID
/// [`ALIGNMENT`]: crate::ALIGNMENT
/// [`DynSizedStructure`]: crate::DynSizedStructure
pub unsafe trait MaybeDynSized: Pointee {
    /// The associated [`Header`] of this tag.
    type Header: Header;

    /// The true base size of the struct without any implicit or additional
    /// padding. Note that `size_of::<T>()` isn't sufficient, as for example
    /// the type could have three `u32` fields, which would add an implicit
    /// `u32` padding. However, this constant **must always** fulfill
    /// `BASE_SIZE >= size_of::<Self::Header>()`.
    ///
    /// The main purpose of this constant is to create awareness when you
    /// implement [`Self::dst_len`], where you should use this. If this value
    /// is correct, we prevent situations where we read uninitialized bytes,
    /// especially when creating tags in builders.
    const BASE_SIZE: usize;

    /// Returns the amount of items in the dynamically sized portion of the
    /// DST. Note that this is not the amount of bytes. So if the dynamically
    /// sized portion is 16 bytes in size and each element is 4 bytes big, then
    /// this function must return 4.
    ///
    /// For sized tags, this just returns `()`. For DSTs, this returns an
    /// `usize`.
    fn dst_len(header: &Self::Header) -> Self::Metadata
    where
        // Either `()` or `usize`, never something else
        Self::Metadata: Default,
    {
        let _ = header;
        Default::default()
    }

    /// Returns the corresponding [`Header`].
    fn header(&self) -> &Self::Header {
        let ptr = &raw const *self;
        // SAFETY: `self` is a valid reference and `Self::Header` is the
        // prefix of this `repr(C)` structure at the same address.
        unsafe { &*ptr.cast::<Self::Header>() }
    }

    /// Returns the payload, i.e., all memory that is not occupied by the
    /// [`Header`] of the type. Implicit trailing padding beyond the
    /// structure size reported in the header is not part of the payload.
    ///
    /// # Panics
    /// Panics if the size reported in the header is smaller than the size of
    /// the [`Header`] itself, which can only happen for oddly formed values.
    fn payload(&self) -> &[u8] {
        let from = size_of::<Self::Header>();
        &self.as_bytes()[from..]
    }

    /// Returns the bytes of the structure, i.e., the header and the payload,
    /// up to the structure size reported by [`Self::header`].
    ///
    /// Implicit trailing padding that the Rust memory layout might add beyond
    /// that size is excluded, as it may be uninitialized for stack-constructed
    /// values and must never be read.
    fn as_bytes(&self) -> &[u8] {
        let ptr = &raw const *self;
        // Clamp to the allocation: a corrupt header size must never cause an
        // out-of-bounds slice.
        let size = self.header().total_size().min(size_of_val(self));
        // SAFETY: `ptr` points to `self`'s allocation, `size` is in bounds,
        // and the first `total_size()` bytes of a value are initialized.
        unsafe { slice::from_raw_parts(ptr.cast::<u8>(), size) }
    }

    /// Returns a pointer to this structure.
    fn as_ptr(&self) -> *const Self::Header {
        self.as_bytes().as_ptr().cast()
    }
}

/// Extension of [`MaybeDynSized`] for Tags.
pub trait Tag: MaybeDynSized {
    /// The ID type that identifies the tag.
    type IDType: PartialEq + Eq;

    /// The ID of this tag. This should be unique across all implementors.
    ///
    /// Although the ID is not yet used in `multiboot2-common`, it ensures
    /// a consistent API in consumer crates.
    const ID: Self::IDType;
}

// This implementation is not needed for parsing but for creation, when
// downstream types just wrap this type.
// SAFETY: `DynSizedStructure` is repr(C) with the header as first field,
// any bit pattern is valid, and `BASE_SIZE`/`dst_len` match the ABI.
unsafe impl<H: Header> MaybeDynSized for DynSizedStructure<H> {
    type Header = H;

    const BASE_SIZE: usize = size_of::<H>();

    fn dst_len(header: &Self::Header) -> Self::Metadata {
        header.payload_len()
    }
}
