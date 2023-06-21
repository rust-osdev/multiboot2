//! Module for the builder-feature.

mod information;
pub(crate) mod traits;

pub use information::InformationBuilder;

use alloc::alloc::alloc;
use alloc::boxed::Box;
use core::alloc::Layout;
use core::mem::{align_of_val, size_of, size_of_val};
use core::ops::Deref;

use crate::{Tag, TagTrait, TagTypeId};

/// Create a boxed tag with the given content.
///
/// # Parameters
/// - `typ` - The given [`TagTypeId`]
/// - `content` - All payload bytes of the DST tag without the tag type or the
///               size. The memory is only read and can be discarded afterwards.
pub(super) fn boxed_dst_tag<T: TagTrait<Metadata = usize> + ?Sized>(
    typ: impl Into<TagTypeId>,
    content: &[u8],
) -> Box<T> {
    // Currently, I do not find a nice way of making this dynamic so that also
    // miri is happy. But it seems that 4 is fine.
    const ALIGN: usize = 4;

    let tag_size = size_of::<TagTypeId>() + size_of::<u32>() + content.len();
    // round up to the next multiple of 8
    // Rust uses this convention for all types. I found out so by checking
    // miris output of the corresponding unit test.
    let alloc_size = (tag_size + 7) & !0b111;
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

    unsafe {
        let boxed = Box::from_raw(raw);
        let reference: &T = boxed.deref();
        // If this panics, please create an issue on GitHub.
        assert_eq!(size_of_val(reference), alloc_size);
        assert_eq!(align_of_val(reference), ALIGN);
        boxed
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

        let tag = boxed_dst_tag::<CustomTag>(tag_type_id, content.as_bytes());
        assert_eq!(tag.typ, tag_type_id);
        assert_eq!(tag.size as usize, METADATA_SIZE + content.len());
        assert_eq!(tag.string(), Ok(content));
    }
}
