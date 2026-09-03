//! Module for [`NetworkTag`].

use crate::{TagHeader, TagType, TagTypeRaw};
use multiboot2_common::{MaybeDynSized, Tag};
use ptr_meta::Pointee;
#[cfg(feature = "builder")]
use {alloc::boxed::Box, multiboot2_common::new_boxed};

/// Contains network information in the form of a DHCP packet.
#[derive(Debug, Pointee)]
#[repr(C, align(8))]
pub struct NetworkTag {
    typ: TagTypeRaw,
    size: u32,
    dhcpack: [u8],
}

impl NetworkTag {
    /// Create a new network tag from the given DHCP package.
    #[cfg(feature = "builder")]
    #[must_use]
    pub fn new(dhcp_pack: &[u8]) -> Box<Self> {
        let header = TagHeader::new(Self::ID, 0 /* filled by new_boxed */);
        new_boxed(header, &[dhcp_pack])
    }
}

// SAFETY: The tag is repr(C) with the header as first field, any bit
// pattern is valid, and `BASE_SIZE`/`dst_len` match the ABI.
unsafe impl MaybeDynSized for NetworkTag {
    type Header = TagHeader;

    const BASE_SIZE: usize = size_of::<TagHeader>();

    fn dst_len(header: &TagHeader) -> usize {
        assert!(header.size as usize >= Self::BASE_SIZE);
        header.size as usize - Self::BASE_SIZE
    }
}

impl Tag for NetworkTag {
    type IDType = TagType;

    const ID: TagType = TagType::Network;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn dst_len_rejects_undersized_header() {
        let header = TagHeader::new(TagType::Network, 4);
        let _ = <NetworkTag as MaybeDynSized>::dst_len(&header);
    }
}
