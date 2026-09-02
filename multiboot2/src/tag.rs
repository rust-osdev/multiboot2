//! Module for the base tag definition [`TagHeader`].

use crate::{TagType, TagTypeRaw};
use core::fmt::Debug;
use multiboot2_common::Header;

/// The common header that all tags have in common. This type is ABI compatible.
///
/// Not to be confused with Multiboot header tags, which are something
/// different.
///
/// It is the sized counterpart of `GenericTag`, an internal type.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(C, align(8))]
pub struct TagHeader {
    /// The ABI-compatible [`TagType`].
    ///
    /// [`TagType`]: crate::TagType
    pub typ: TagTypeRaw, /* u32 */
    /// The total size of the tag including the header.
    pub size: u32,
    // Followed by optional additional tag specific fields.
}

impl TagHeader {
    /// Creates a new header.
    pub fn new(typ: impl Into<TagType>, size: u32) -> Self {
        Self {
            typ: TagTypeRaw::new(typ.into().val()),
            size,
        }
    }
}

// SAFETY: The header is a padding-free repr(C) struct of raw integers, and
// any bit pattern is valid for it.
unsafe impl Header for TagHeader {
    fn total_size(&self) -> usize {
        self.size as usize
    }

    fn set_size(&mut self, total_size: usize) {
        self.size = total_size as u32
    }
}
