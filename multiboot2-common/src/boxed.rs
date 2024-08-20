//! Module for [`new_boxed`].

use crate::{increase_to_alignment, Header, MaybeDynSized, ALIGNMENT};
use alloc::boxed::Box;
use core::alloc::Layout;
use core::mem;
use core::ops::Deref;
use core::ptr;

/// Creates a new tag implementing [`MaybeDynSized`] on the heap. This works for
/// sized and unsized tags. However, it only makes sense to use this for tags
/// that are DSTs (unsized). For regular sized structs, you can just create a
/// typical constructor and box the result.
///
/// The provided `header`' total size (see [`Header`]) will be set dynamically
/// by this function using [`Header::set_size`]. However, it must contain all
/// other relevant metadata or update it in the `set_size` callback.
///
/// # Parameters
/// - `additional_bytes_slices`: Array of byte slices that should be included
///   without additional padding in-between. You don't need to add the bytes
///   for [`Header`], but only additional payload.
#[must_use]
pub fn new_boxed<T: MaybeDynSized<Metadata = usize> + ?Sized>(
    mut header: T::Header,
    additional_bytes_slices: &[&[u8]],
) -> Box<T> {
    let additional_size = additional_bytes_slices
        .iter()
        .map(|b| b.len())
        .sum::<usize>();

    let tag_size = mem::size_of::<T::Header>() + additional_size;
    header.set_size(tag_size);

    // Allocation size is multiple of alignment.
    // See <https://doc.rust-lang.org/reference/type-layout.html>
    let alloc_size = increase_to_alignment(tag_size);
    let layout = Layout::from_size_align(alloc_size, ALIGNMENT).unwrap();
    let heap_ptr = unsafe { alloc::alloc::alloc(layout) };
    assert!(!heap_ptr.is_null());

    // write header
    {
        let len = mem::size_of::<T::Header>();
        let ptr = core::ptr::addr_of!(header);
        unsafe {
            ptr::copy_nonoverlapping(ptr.cast::<u8>(), heap_ptr, len);
        }
    }

    // write body
    {
        let mut write_offset = mem::size_of::<T::Header>();
        for &bytes in additional_bytes_slices {
            let len = bytes.len();
            let src = bytes.as_ptr();
            unsafe {
                let dst = heap_ptr.add(write_offset);
                ptr::copy_nonoverlapping(src, dst, len);
                write_offset += len;
            }
        }
    }

    // This is a fat pointer for DSTs and a thin pointer for sized `T`s.
    let ptr: *mut T = ptr_meta::from_raw_parts_mut(heap_ptr.cast(), T::dst_len(&header));
    let reference = unsafe { Box::from_raw(ptr) };

    // If this panic triggers, there is a fundamental flaw in my logic. This is
    // not the fault of an API user.
    assert_eq!(
        mem::size_of_val(reference.deref()),
        alloc_size,
        "Allocation should match Rusts expectation"
    );

    reference
}

/// Clones a [`MaybeDynSized`] by calling [`new_boxed`].
#[must_use]
pub fn clone_dyn<T: MaybeDynSized<Metadata = usize> + ?Sized>(tag: &T) -> Box<T> {
    new_boxed(tag.header().clone(), &[tag.payload()])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{DummyDstTag, DummyTestHeader};
    use crate::Tag;

    #[test]
    fn test_new_boxed() {
        let header = DummyTestHeader::new(DummyDstTag::ID, 0);
        let tag = new_boxed::<DummyDstTag>(header, &[&[0, 1, 2, 3]]);
        assert_eq!(tag.header().typ(), 42);
        assert_eq!(tag.payload(), &[0, 1, 2, 3]);

        // Test that bytes are added consecutively without gaps.
        let header = DummyTestHeader::new(0xdead_beef, 0);
        let tag = new_boxed::<DummyDstTag>(header, &[&[0], &[1], &[2, 3]]);
        assert_eq!(tag.header().typ(), 0xdead_beef);
        assert_eq!(tag.payload(), &[0, 1, 2, 3]);
    }

    #[test]
    fn test_clone_tag() {
        let header = DummyTestHeader::new(DummyDstTag::ID, 0);
        let tag = new_boxed::<DummyDstTag>(header, &[&[0, 1, 2, 3]]);
        assert_eq!(tag.header().typ(), 42);
        assert_eq!(tag.payload(), &[0, 1, 2, 3]);

        let _cloned = clone_dyn(tag.as_ref());
    }
}
