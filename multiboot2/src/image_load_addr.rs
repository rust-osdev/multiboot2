//! Module for [`ImageLoadPhysAddrTag`].

use crate::TagType;
use crate::tag::TagHeader;
use multiboot2_common::{MaybeDynSized, Tag};

/// The physical load address tag. Typically, this is only available if the
/// binary was relocated, for example if the relocatable header tag was
/// specified.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C, align(8))]
pub struct ImageLoadPhysAddrTag {
    header: TagHeader,
    load_base_addr: u32,
}

impl ImageLoadPhysAddrTag {
    /// Constructs a new tag.
    #[must_use]
    pub fn new(load_base_addr: u32) -> Self {
        Self {
            header: TagHeader::new(Self::ID, Self::BASE_SIZE as u32),
            load_base_addr,
        }
    }

    /// Returns the load base address.
    #[must_use]
    pub const fn load_base_addr(&self) -> u32 {
        self.load_base_addr
    }
}
// SAFETY: The tag is repr(C) with the header as first field, any bit
// pattern is valid, and `BASE_SIZE`/`dst_len` match the ABI.
unsafe impl MaybeDynSized for ImageLoadPhysAddrTag {
    type Header = TagHeader;

    const BASE_SIZE: usize = size_of::<TagHeader>() + size_of::<u32>();
}

impl Tag for ImageLoadPhysAddrTag {
    type IDType = TagType;

    const ID: TagType = TagType::LoadBaseAddr;
}

#[cfg(all(test, feature = "builder"))]
mod tests {
    use super::ImageLoadPhysAddrTag;

    const ADDR: u32 = 0xABCDEF;

    /// The tag must report the spec-mandated size (12), not the padded Rust
    /// type size (16).
    #[test]
    fn base_size_excludes_trailing_padding() {
        use multiboot2_common::MaybeDynSized;
        assert_eq!(<ImageLoadPhysAddrTag as MaybeDynSized>::BASE_SIZE, 12);
    }

    #[test]
    fn test_build_load_addr() {
        let tag = ImageLoadPhysAddrTag::new(ADDR);
        assert_eq!(tag.load_base_addr(), ADDR);
    }
}
