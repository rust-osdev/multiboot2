//! Module for the traits [`MaybeDynSized`] and [`Tag`].

use crate::{BytesRef, DynSizedStructure, Header};
use core::mem;
use core::slice;
use ptr_meta::Pointee;

/// A trait to abstract sized and unsized structures (DSTs). It enables
/// casting a [`DynSizedStructure`] to sized or unsized structures using
/// [`DynSizedStructure::cast`].
///
/// Structs that are a DST must provide a **correct**
/// [`MaybeDynSized::dst_len`] implementation.
///
/// [`ID`]: Tag::ID
/// [`DynSizedStructure`]: crate::DynSizedStructure
pub trait MaybeDynSized: Pointee {
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
    fn dst_len(header: &Self::Header) -> Self::Metadata;

    /// Returns the corresponding [`Header`].
    fn header(&self) -> &Self::Header {
        let ptr = core::ptr::addr_of!(*self);
        unsafe { &*ptr.cast::<Self::Header>() }
    }

    /// Returns the payload, i.e., all memory that is not occupied by the
    /// [`Header`] of the type.
    fn payload(&self) -> &[u8] {
        let from = mem::size_of::<Self::Header>();
        &self.as_bytes()[from..]
    }

    /// Returns the whole allocated bytes for this structure encapsulated in
    /// [`BytesRef`]. This includes padding bytes. To only get the "true" tag
    /// data, read the tag size from [`Self::header`] and create a sub slice.
    fn as_bytes(&self) -> BytesRef<Self::Header> {
        let ptr = core::ptr::addr_of!(*self);
        // Actual tag size, optionally with terminating padding.
        let size = mem::size_of_val(self);
        let slice = unsafe { slice::from_raw_parts(ptr.cast::<u8>(), size) };
        // Unwrap is fine as this type can't exist without the underlying memory
        // guarantees.
        BytesRef::try_from(slice).unwrap()
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

impl<H: Header> MaybeDynSized for DynSizedStructure<H> {
    type Header = H;

    const BASE_SIZE: usize = mem::size_of::<H>();

    fn dst_len(header: &Self::Header) -> Self::Metadata {
        header.payload_len()
    }
}
