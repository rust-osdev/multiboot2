//! Module for [`ImageLoadPhysAddrTag`].

use crate::tag::TagHeader;
use crate::{Tag, TagTrait, TagType};
#[cfg(feature = "builder")]
use core::mem::size_of;

/// The physical load address tag. Typically, this is only available if the
/// binary was relocated, for example if the relocatable header tag was
/// specified.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct ImageLoadPhysAddrTag {
    header: TagHeader,
    load_base_addr: u32,
}

impl ImageLoadPhysAddrTag {
    #[cfg(feature = "builder")]
    #[must_use]
    pub fn new(load_base_addr: u32) -> Self {
        Self {
            header: TagHeader::new(Self::ID, size_of::<Self>().try_into().unwrap()),
            load_base_addr,
        }
    }

    /// Returns the load base address.
    #[must_use]
    pub const fn load_base_addr(&self) -> u32 {
        self.load_base_addr
    }
}

impl TagTrait for ImageLoadPhysAddrTag {
    const ID: TagType = TagType::LoadBaseAddr;

    fn dst_size(_base_tag: &Tag) {}
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
