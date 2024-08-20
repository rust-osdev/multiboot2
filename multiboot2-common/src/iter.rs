//! Iterator over Multiboot2 structures. Technically, the process for iterating
//! Multiboot2 information tags and iterating Multiboot2 header tags is the
//! same.

use crate::{increase_to_alignment, DynSizedStructure, Header, ALIGNMENT};
use core::marker::PhantomData;
use core::mem;

/// Iterates over the tags (modelled by [`DynSizedStructure`]) of the underlying
/// byte slice. Each tag is expected to have the same common [`Header`].
///
/// As the iterator emits elements of type [`DynSizedStructure`], users should
/// casted them to specific [`Tag`]s using [`DynSizedStructure::cast`] following
/// a user policy. This can for example happen on the basis of some ID.
///
/// This type ensures the memory safety guarantees promised by this crates
/// documentation.
///
/// [`Tag`]: crate::Tag
#[derive(Clone, Debug)]
pub struct TagIter<'a, H: Header> {
    /// Absolute offset to next tag and updated in each iteration.
    next_tag_offset: usize,
    buffer: &'a [u8],
    // Ensure that all instances are bound to a specific `Header`.
    // Otherwise, UB can happen.
    _t: PhantomData<H>,
}

impl<'a, H: Header> TagIter<'a, H> {
    /// Creates a new iterator.
    #[must_use]
    pub fn new(mem: &'a [u8]) -> Self {
        // Assert alignment.
        assert_eq!(mem.as_ptr().align_offset(ALIGNMENT), 0);

        TagIter {
            next_tag_offset: 0,
            buffer: mem,
            _t: PhantomData,
        }
    }
}

impl<'a, H: Header + 'a> Iterator for TagIter<'a, H> {
    type Item = &'a DynSizedStructure<H>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_tag_offset == self.buffer.len() {
            return None;
        }
        assert!(self.next_tag_offset < self.buffer.len());

        let ptr = unsafe { self.buffer.as_ptr().add(self.next_tag_offset) }.cast::<H>();
        let tag_hdr = unsafe { &*ptr };

        // Get relevant byte portion for the next tag. This includes padding
        // bytes to fulfill Rust memory guarantees. Otherwise, Miri complains.
        // See <https://doc.rust-lang.org/reference/type-layout.html>.
        let slice = {
            let from = self.next_tag_offset;
            let len = mem::size_of::<H>() + tag_hdr.payload_len();
            let to = from + len;

            // The size of (the allocation for) a value is always a multiple of
            // its alignment.
            // https://doc.rust-lang.org/reference/type-layout.html
            let to = increase_to_alignment(to);

            // Update ptr for next iteration.
            self.next_tag_offset += to - from;

            &self.buffer[from..to]
        };

        // unwrap: We should not fail at this point.
        let tag = DynSizedStructure::ref_from_slice(slice).unwrap();
        Some(tag)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::{AlignedBytes, DummyTestHeader};
    use crate::TagIter;
    use core::borrow::Borrow;

    #[test]
    fn test_tag_iter() {
        #[rustfmt::skip]
        let bytes = AlignedBytes::new(
            [
                /* Some minimal tag.  */
                0xff, 0, 0, 0,
                8, 0, 0, 0,
                /* Some tag with payload.  */
                0xfe, 0, 0, 0,
                12, 0, 0, 0,
                1, 2, 3, 4,
                // Padding
                0, 0, 0, 0,
                /* End tag */
                0, 0, 0, 0,
                8, 0, 0, 0,
            ],
        );
        let mut iter = TagIter::<DummyTestHeader>::new(bytes.borrow());
        let first = iter.next().unwrap();
        assert_eq!(first.header().typ(), 0xff);
        assert_eq!(first.header().size(), 8);
        assert!(first.payload().is_empty());

        let second = iter.next().unwrap();
        assert_eq!(second.header().typ(), 0xfe);
        assert_eq!(second.header().size(), 12);
        assert_eq!(&second.payload(), &[1, 2, 3, 4]);

        let third = iter.next().unwrap();
        assert_eq!(third.header().typ(), 0);
        assert_eq!(third.header().size(), 8);
        assert!(first.payload().is_empty());

        assert_eq!(iter.next(), None);
    }
}
