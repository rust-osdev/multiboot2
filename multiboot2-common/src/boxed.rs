//! Module for [`new_boxed`].

use crate::{ALIGNMENT, Header, MaybeDynSized, increase_to_alignment};
use alloc::boxed::Box;
use core::alloc::Layout;
use core::ops::Deref;
use core::ptr;

/// Creates a new tag implementing [`MaybeDynSized`] on the heap.
///
/// This works for sized and unsized tags. However, it only makes sense to use
/// this for tags that are DSTs (unsized). For regular sized structs, you can
/// just create a typical constructor and box the result.
///
/// The provided `header`' total size (see [`Header`]) will be set dynamically
/// by this function using [`Header::set_size`]. However, it must contain all
/// other relevant metadata or update it in the `set_size` callback.
///
/// # Requirements
///
/// `T` must uphold the requirements of [`MaybeDynSized`], in particular a
/// correct [`MaybeDynSized::BASE_SIZE`] and [`MaybeDynSized::dst_len`]
/// implementation. These requirements ensure that the allocation made here
/// matches the layout of `T`.
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

    let tag_size = size_of::<T::Header>() + additional_size;
    header.set_size(tag_size);
    // Protect against incorrect set_size() implementations:
    assert_eq!(
        header.total_size(),
        tag_size,
        "the reported size should round-trip through the header"
    );

    // Allocation size is multiple of alignment.
    // See <https://doc.rust-lang.org/reference/type-layout.html>
    let alloc_size = increase_to_alignment(tag_size);
    let layout = Layout::from_size_align(alloc_size, ALIGNMENT).unwrap();
    // Use a zeroed allocation so that the trailing padding in
    // `[tag_size, alloc_size)` is initialized. The header and body writes
    // below only cover `[0, tag_size)`; without zeroing, reading the padding
    // through the safe `MaybeDynSized::as_bytes`/`payload` accessors would be
    // undefined behavior. The Multiboot2 spec also mandates zero padding.
    // SAFETY: `layout` matches the requested allocation size and alignment.
    let heap_ptr = unsafe { alloc::alloc::alloc_zeroed(layout) };
    assert!(!heap_ptr.is_null());

    // write header
    {
        let len = size_of::<T::Header>();
        let ptr = &raw const header;
        // SAFETY: `header` is a fully initialized stack value and `heap_ptr`
        // points into the freshly allocated destination buffer.
        unsafe {
            ptr::copy_nonoverlapping(ptr.cast::<u8>(), heap_ptr, len);
        }
    }

    // write body
    {
        let mut write_offset = size_of::<T::Header>();
        for &bytes in additional_bytes_slices {
            let len = bytes.len();
            let src = bytes.as_ptr();
            let dst = heap_ptr.wrapping_add(write_offset);
            // SAFETY: `src` is a valid slice and `dst` stays inside the
            // allocated object without overlapping `src`.
            unsafe {
                ptr::copy_nonoverlapping(src, dst, len);
            }
            write_offset += len;
        }
    }

    // This is a fat pointer for DSTs and a thin pointer for sized `T`s.
    // SAFETY: The allocation was sized for `T` and all bytes up to the
    // reported dynamic length were initialized above.
    let ptr: *mut T = ptr_meta::from_raw_parts_mut(heap_ptr.cast(), T::dst_len(&header));
    // SAFETY: `ptr` points to the initialized allocation described above.
    let reference = unsafe { Box::from_raw(ptr) };

    // If this panic triggers, there is a fundamental flaw in my logic. This is
    // not the fault of an API user.
    assert_eq!(
        size_of_val(reference.deref()),
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
    use crate::Tag;
    use crate::test_utils::{DummyDstTag, DummyTestHeader};
    use core::slice;

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
    fn test_new_boxed_zeroes_padding() {
        // A payload of 1 byte yields a 9-byte tag in a 16-byte allocation.
        // `as_bytes()` must exclude the 7 trailing padding bytes, while the
        // allocation itself must be zeroed there (guaranteed by
        // `alloc_zeroed`), as the Multiboot2 spec mandates zeroed padding
        // between tags.
        let header = DummyTestHeader::new(DummyDstTag::ID, 0);
        let tag = new_boxed::<DummyDstTag>(header, &[&[0xff]]);
        assert_eq!(tag.as_bytes().len(), 9);
        let ptr = (&raw const *tag).cast::<u8>();
        // SAFETY: The allocation spans `size_of_val` bytes and is fully
        // initialized by `new_boxed` (zeroed allocation).
        let all_bytes = unsafe { slice::from_raw_parts(ptr, size_of_val(&*tag)) };
        assert_eq!(all_bytes.len(), 16);
        assert_eq!(&all_bytes[9..16], &[0, 0, 0, 0, 0, 0, 0]);
    }

    /// Header whose size field is artificially small, mimicking a lossy
    /// `set_size` implementation without needing a huge allocation.
    #[derive(Clone, Debug, PartialEq, Eq)]
    #[repr(C)]
    struct TinySizeHeader {
        size: u8,
        _pad: [u8; 7],
    }

    // SAFETY: The header is a padding-free repr(C) struct of raw integers,
    // and any bit pattern is valid for it.
    unsafe impl crate::Header for TinySizeHeader {
        fn total_size(&self) -> usize {
            self.size as usize
        }

        fn set_size(&mut self, total_size: usize) {
            self.size = total_size as u8;
        }
    }

    #[test]
    #[should_panic(expected = "round-trip")]
    fn test_new_boxed_rejects_lossy_set_size() {
        // A total size the header can't store must cause a panic before the
        // allocation happens. Continuing with a truncated size would create a
        // `Box` whose layout disagrees with the allocation, which is
        // undefined behavior when the `Box` is deallocated.
        let header = TinySizeHeader {
            size: 0,
            _pad: [0; 7],
        };
        let _ = new_boxed::<crate::DynSizedStructure<TinySizeHeader>>(header, &[&[0_u8; 256]]);
    }

    #[test]
    fn test_clone_tag() {
        // A 5-byte payload, so that the reported tag size (13) is no
        // multiple of the alignment.
        let header = DummyTestHeader::new(DummyDstTag::ID, 0);
        let tag = new_boxed::<DummyDstTag>(header, &[&[0, 1, 2, 3, 4]]);
        assert_eq!(tag.header().typ(), 42);
        assert_eq!(tag.payload(), &[0, 1, 2, 3, 4]);

        let cloned = clone_dyn(tag.as_ref());
        // The clone must round-trip exactly; especially, the reported size
        // must not grow to the padded allocation size.
        assert_eq!(cloned.header(), tag.header());
        assert_eq!(cloned.payload(), tag.payload());
    }
}
