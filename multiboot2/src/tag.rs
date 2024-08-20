//! Module for the base tag definition [`TagHeader`].

use crate::TagTypeId;
use core::fmt::Debug;
use core::mem;
use multiboot2_common::Header;

/// The common header that all tags have in common. This type is ABI compatible.
///
/// Not to be confused with Multiboot header tags, which are something
/// different.
///
/// It is the sized counterpart of `GenericTag`, an internal type.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(C, align(8))] // Alignment also propagates to all tag types using this.
pub struct TagHeader {
    /// The ABI-compatible [`TagType`].
    ///
    /// [`TagType`]: crate::TagType
    pub typ: TagTypeId, /* u32 */
    /// The total size of the tag including the header.
    pub size: u32,
    // Followed by optional additional tag specific fields.
}

impl TagHeader {
    /// Creates a new header.
    pub fn new(typ: impl Into<TagTypeId>, size: u32) -> Self {
        Self {
            typ: typ.into(),
            size,
        }
    }
}

impl Header for TagHeader {
    fn payload_len(&self) -> usize {
        assert!(self.size as usize >= mem::size_of::<Self>());
        self.size as usize - mem::size_of::<Self>()
    }

    fn set_size(&mut self, total_size: usize) {
        self.size = total_size as u32
    }
}
