//! Module for [`TagTrait`].

use crate::tag::TagHeader;
use crate::TagType;
use ptr_meta::Pointee;

/// A trait to abstract over all sized and unsized tags (DSTs). For sized tags,
/// this trait does not much. For DSTs, a [`TagTrait::dst_len`] implementation
/// must be provided, which returns the right size hint for the dynamically
/// sized portion of the struct.
///
/// # Trivia
/// This crate uses the [`Pointee`]-abstraction of the [`ptr_meta`] crate to
/// create fat pointers for tags that are DST.
pub trait TagTrait: Pointee {
    /// The numeric ID of this tag.
    const ID: TagType;

    /// Returns the amount of items in the dynamically sized portion of the
    /// DST. Note that this is not the amount of bytes. So if the dynamically
    /// sized portion is 16 bytes in size and each element is 4 bytes big, then
    /// this function must return 4.
    ///
    /// For sized tags, this just returns `()`. For DSTs, this returns an
    /// `usize`.
    fn dst_len(header: &TagHeader) -> Self::Metadata;

    /// Returns the tag as the common base tag structure.
    fn as_base_tag(&self) -> &TagHeader {
        let ptr = core::ptr::addr_of!(*self);
        unsafe { &*ptr.cast::<TagHeader>() }
    }

    /// Returns the total size of the tag. The depends on the `size` field of
    /// the tag.
    fn size(&self) -> usize {
        self.as_base_tag().size as usize
    }

    /// Returns a slice to the underlying bytes of the tag. This includes all
    /// bytes, also for tags that are DSTs. The slice length depends on the
    /// `size` field of the tag.
    fn as_bytes(&self) -> &[u8] {
        let ptr = core::ptr::addr_of!(*self);
        unsafe { core::slice::from_raw_parts(ptr.cast(), self.size()) }
    }
}
