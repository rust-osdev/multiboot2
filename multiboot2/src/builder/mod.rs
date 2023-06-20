//! Module for the builder-feature.

mod information;
pub(crate) mod traits;

pub use information::InformationBuilder;

use alloc::alloc::alloc;
use alloc::boxed::Box;
use core::alloc::Layout;
use core::mem::size_of;

use crate::{TagTrait, TagTypeId};

/// Create a boxed tag with the given content.
pub(super) fn boxed_dst_tag<T: TagTrait<Metadata = usize> + ?Sized>(
    typ: impl Into<TagTypeId>,
    content: &[u8],
) -> Box<T> {
    // based on https://stackoverflow.com/a/64121094/2192464
    let (layout, size_offset) = Layout::new::<TagTypeId>()
        .extend(Layout::new::<u32>())
        .unwrap();
    let (layout, inner_offset) = layout
        .extend(Layout::array::<usize>(content.len()).unwrap())
        .unwrap();
    let ptr = unsafe { alloc(layout) };
    assert!(!ptr.is_null());
    unsafe {
        // initialize the content as good as we can
        ptr.cast::<TagTypeId>().write(typ.into());
        ptr.add(size_offset).cast::<u32>().write(
            (content.len() + size_of::<TagTypeId>() + size_of::<u32>())
                .try_into()
                .unwrap(),
        );
        // initialize body
        let content_ptr = ptr.add(inner_offset);
        for (idx, val) in content.iter().enumerate() {
            content_ptr.add(idx).write(*val);
        }
        Box::from_raw(ptr_meta::from_raw_parts_mut(ptr as *mut (), content.len()))
    }
}
