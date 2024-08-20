//! Module for [`ImageLoadPhysAddrTag`].

use crate::tag::TagHeader;
use crate::TagType;
#[cfg(feature = "builder")]
use core::mem::size_of;
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
    const BASE_SIZE: usize = size_of::<TagHeader>() + size_of::<u32>();

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
impl MaybeDynSized for ImageLoadPhysAddrTag {
    type Header = TagHeader;

    const BASE_SIZE: usize = size_of::<Self>();

    fn dst_len(_: &TagHeader) {}
}

impl Tag for ImageLoadPhysAddrTag {
    type IDType = TagType;

    const ID: TagType = TagType::LoadBaseAddr;
}

#[cfg(all(test, feature = "builder"))]
mod tests {
    use super::ImageLoadPhysAddrTag;

    const ADDR: u32 = 0xABCDEF;

    #[test]
    fn test_build_load_addr() {
        let tag = ImageLoadPhysAddrTag::new(ADDR);
        assert_eq!(tag.load_base_addr(), ADDR);
    }
}
