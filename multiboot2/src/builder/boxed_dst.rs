//! Module for [`BoxedDst`].

use crate::util::increase_to_alignment;
use crate::{TagHeader, TagTrait, TagTypeId};
use alloc::alloc::alloc;
use core::alloc::Layout;
use core::marker::PhantomData;
use core::mem::size_of;
use core::ops::Deref;
use core::ptr::NonNull;

/// A helper type to create boxed DST, i.e., tags with a dynamic size for the
/// builder. This is tricky in Rust. This type behaves similar to the regular
/// `Box` type except that it ensure the same layout is used for the (explicit)
/// allocation and the (implicit) deallocation of memory. Otherwise, I didn't
/// find any way to figure out the right layout for a DST. Miri always reported
/// issues that the deallocation used a wrong layout.
///
/// Technically, I'm certain this code is memory safe. But with this type, I
/// also can convince miri that it is.
#[derive(Debug, Eq)]
pub struct BoxedDst<T: ?Sized> {
    ptr: core::ptr::NonNull<T>,
    layout: Layout,
    // marker: I used this only as the regular Box impl also does it.
    _marker: PhantomData<T>,
}

impl<T: TagTrait<Metadata = usize> + ?Sized> BoxedDst<T> {
    /// Create a boxed tag with the given content.
    ///
    /// # Parameters
    /// - `content` - All payload bytes of the DST tag without the tag type or
    ///               the size. The memory is only read and can be discarded
    ///               afterwards.
    pub(crate) fn new(content: &[u8]) -> Self {
        // Currently, I do not find a nice way of making this dynamic so that
        // also miri is guaranteed to be happy. But it seems that 4 is fine
        // here. I do have control over allocation and deallocation.
        const ALIGN: usize = 4;

        let tag_size = size_of::<TagTypeId>() + size_of::<u32>() + content.len();

        // The size of [the allocation for] a value is always a multiple of its
        // alignment.
        // https://doc.rust-lang.org/reference/type-layout.html
        let alloc_size = increase_to_alignment(tag_size);
        let layout = Layout::from_size_align(alloc_size, ALIGN).unwrap();
        let ptr = unsafe { alloc(layout) };
        assert!(!ptr.is_null());

        // write tag content to memory
        unsafe {
            // write tag type
            let ptrx = ptr.cast::<TagTypeId>();
            ptrx.write(T::ID.into());

            // write tag size
            let ptrx = ptrx.add(1).cast::<u32>();
            ptrx.write(tag_size as u32);

            // write rest of content
            let ptrx = ptrx.add(1).cast::<u8>();
            let tag_content_slice = core::slice::from_raw_parts_mut(ptrx, content.len());
            for (i, &byte) in content.iter().enumerate() {
                tag_content_slice[i] = byte;
            }
        }

        let base_tag = unsafe { &*ptr.cast::<TagHeader>() };
        let raw: *mut T = ptr_meta::from_raw_parts_mut(ptr.cast(), T::dst_len(base_tag));

        Self {
            ptr: NonNull::new(raw).unwrap(),
            layout,
            _marker: PhantomData,
        }
    }
}

impl<T: ?Sized> Drop for BoxedDst<T> {
    fn drop(&mut self) {
        unsafe { alloc::alloc::dealloc(self.ptr.as_ptr().cast(), self.layout) }
    }
}

impl<T: ?Sized> Deref for BoxedDst<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T: ?Sized + PartialEq> PartialEq for BoxedDst<T> {
    fn eq(&self, other: &Self) -> bool {
        self.deref().eq(other.deref())
    }
}

#[cfg(test)]
#[cfg(not(miri))]
mod tests {
    use super::*;
    use crate::test_util::AlignedBytes;
    use crate::{parse_slice_as_string, StringError, TagHeader, TagType};
    use core::borrow::Borrow;

    const METADATA_SIZE: usize = 8;

    #[derive(ptr_meta::Pointee)]
    #[repr(C)]
    struct CustomTag {
        typ: TagTypeId,
        size: u32,
        string: [u8],
    }

    impl CustomTag {
        fn string(&self) -> Result<&str, StringError> {
            parse_slice_as_string(&self.string)
        }
    }

    impl TagTrait for CustomTag {
        const ID: TagType = TagType::Custom(0x1337);

        fn dst_len(header: &TagHeader) -> usize {
            assert!(header.size as usize >= METADATA_SIZE);
            header.size as usize - METADATA_SIZE
        }
    }

    #[test]
    fn test_boxed_dst_tag() {
        let content = AlignedBytes::new(*b"hallo\0");
        let content_rust_str = "hallo";
        let tag = BoxedDst::<CustomTag>::new(content.borrow());
        assert_eq!(tag.typ, CustomTag::ID);
        assert_eq!(tag.size as usize, METADATA_SIZE + content.len());
        assert_eq!(tag.string(), Ok(content_rust_str));
    }

    #[test]
    fn can_hold_tag_trait() {
        const fn consume<T: TagTrait + ?Sized>(_: &T) {}
        let content = AlignedBytes::new(*b"hallo\0");
        let tag = BoxedDst::<CustomTag>::new(content.borrow());
        consume(tag.deref());
        consume(&*tag);
    }
}
