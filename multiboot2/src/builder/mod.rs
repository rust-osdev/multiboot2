//! Module for the builder-feature.

mod information;
pub(crate) mod traits;

pub use information::InformationBuilder;

use alloc::alloc::alloc;
use core::alloc::Layout;
use core::marker::PhantomData;
use core::mem::size_of;
use core::ops::Deref;
use core::ptr::NonNull;

use crate::{Tag, TagTrait, TagTypeId};

/// A helper type to create boxed DST, i.e., tags with a dynamic size for the
/// builder. This is tricky in Rust. This type behaves similar to the regular
/// `Box` type except that it ensure the same layout is used for the (explicit)
/// allocation and the (implicit) deallocation of memory. Otherwise, I didn't
/// found any way to figure out the right layout for a DST. Miri always reported
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
    /// - `typ` - The given [`TagTypeId`]
    /// - `content` - All payload bytes of the DST tag without the tag type or
    ///               the size. The memory is only read and can be discarded
    ///               afterwards.
    pub(crate) fn new(typ: impl Into<TagTypeId>, content: &[u8]) -> Self {
        // Currently, I do not find a nice way of making this dynamic so that
        // also miri is guaranteed to be happy. But it seems that 4 is fine
        // here. I do have control over allocation and deallocation.
        const ALIGN: usize = 4;

        let tag_size = size_of::<TagTypeId>() + size_of::<u32>() + content.len();

        // By using miri, I could figure out that there often are problems where
        // miri thinks an allocation is smaller then necessary. Most probably
        // due to not packed structs. Using packed structs however
        // (especially with DSTs), is a crazy ass pain and unusable :/ Therefore,
        // the best solution I can think of is to allocate a few byte more than
        // necessary. I think that during runtime, everything works fine and
        // that no memory issues are present.
        let alloc_size = (tag_size + 7) & !7; // align to next 8 byte boundary
        let layout = Layout::from_size_align(alloc_size, ALIGN).unwrap();
        let ptr = unsafe { alloc(layout) };
        assert!(!ptr.is_null());

        // write tag content to memory
        unsafe {
            // write tag type
            let ptrx = ptr.cast::<TagTypeId>();
            ptrx.write(typ.into());

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

        let base_tag = unsafe { &*ptr.cast::<Tag>() };
        let raw: *mut T = ptr_meta::from_raw_parts_mut(ptr.cast(), T::dst_size(base_tag));

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
mod tests {
    use super::*;

    const METADATA_SIZE: usize = 8;

    #[derive(ptr_meta::Pointee)]
    #[repr(C)]
    struct CustomTag {
        typ: TagTypeId,
        size: u32,
        string: [u8],
    }

    impl CustomTag {
        fn string(&self) -> Result<&str, core::str::Utf8Error> {
            Tag::get_dst_str_slice(&self.string)
        }
    }

    impl TagTrait for CustomTag {
        fn dst_size(base_tag: &Tag) -> usize {
            assert!(base_tag.size as usize >= METADATA_SIZE);
            base_tag.size as usize - METADATA_SIZE
        }
    }

    #[test]
    fn test_boxed_dst_tag() {
        let tag_type_id = 1337_u32;
        let content = "hallo";

        let tag = unsafe { BoxedDst::<CustomTag>::new(tag_type_id, content.as_bytes()) };
        assert_eq!(tag.typ, tag_type_id);
        assert_eq!(tag.size as usize, METADATA_SIZE + content.len());
        assert_eq!(tag.string(), Ok(content));
    }
}
