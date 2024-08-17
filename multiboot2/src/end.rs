//! Module for [`EndTag`].

use crate::{TagHeader, TagTrait, TagType, TagTypeId};

/// The end tag ends the information struct.
#[derive(Debug)]
#[repr(C, align(8))]
pub struct EndTag {
    typ: TagTypeId,
    size: u32,
}

impl Default for EndTag {
    fn default() -> Self {
        Self {
            typ: TagType::End.into(),
            size: 8,
        }
    }
}

impl TagTrait for EndTag {
    const ID: TagType = TagType::End;

    fn dst_len(_: &TagHeader) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Compile time test for [`EndTag`].
    fn test_end_tag_size() {
        unsafe {
            core::mem::transmute::<[u8; 8], EndTag>([0u8; 8]);
        }
    }
}
