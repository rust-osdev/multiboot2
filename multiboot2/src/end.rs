//! Module for [`EndTag`].

use crate::{TagHeader, TagType};
use multiboot2_common::{MaybeDynSized, Tag};

/// The end tag ends the information struct.
#[derive(Debug)]
#[repr(C, align(8))]
pub struct EndTag {
    header: TagHeader,
}

impl Default for EndTag {
    fn default() -> Self {
        Self {
            header: TagHeader::new(Self::ID, size_of::<Self>() as u32),
        }
    }
}

impl MaybeDynSized for EndTag {
    type Header = TagHeader;

    const BASE_SIZE: usize = size_of::<Self>();
}

impl Tag for EndTag {
    type IDType = TagType;

    const ID: TagType = TagType::End;
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::transmute;

    #[test]
    /// Compile time test for [`EndTag`].
    fn test_end_tag_size() {
        // SAFETY: `EndTag` is a plain `repr(C)` POD with the exact
        // eight-byte end-tag layout.
        unsafe {
            transmute::<[u8; 8], EndTag>([0u8; 8]);
        }
    }
}
