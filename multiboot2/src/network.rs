//! Module for [`NetworkTag`].

use crate::{TagHeader, TagType, TagTypeId};
use core::mem;
use multiboot2_common::{MaybeDynSized, Tag};
use ptr_meta::Pointee;
#[cfg(feature = "builder")]
use {alloc::boxed::Box, multiboot2_common::new_boxed};

/// The end tag ends the information struct.
#[derive(Debug, Pointee)]
#[repr(C, align(8))]
pub struct NetworkTag {
    typ: TagTypeId,
    size: u32,
    dhcpack: [u8],
}

impl NetworkTag {
    /// Create a new network tag from the given DHCP package.
    #[cfg(feature = "builder")]
    #[must_use]
    pub fn new(dhcp_pack: &[u8]) -> Box<Self> {
        let header = TagHeader::new(Self::ID, 0);
        new_boxed(header, &[dhcp_pack])
    }
}

impl MaybeDynSized for NetworkTag {
    type Header = TagHeader;

    const BASE_SIZE: usize = mem::size_of::<TagHeader>();

    fn dst_len(header: &TagHeader) -> usize {
        header.size as usize - Self::BASE_SIZE
    }
}

impl Tag for NetworkTag {
    type IDType = TagType;

    const ID: TagType = TagType::Network;
}
