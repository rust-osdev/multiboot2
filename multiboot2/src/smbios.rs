//! Module for [`SmbiosTag`].

use crate::tag::TagHeader;
use crate::TagType;
use core::fmt::Debug;
use core::mem;
use multiboot2_common::{MaybeDynSized, Tag};
#[cfg(feature = "builder")]
use {alloc::boxed::Box, multiboot2_common::new_boxed};

/// This tag contains a copy of SMBIOS tables as well as their version.
#[derive(ptr_meta::Pointee, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C, align(8))]
pub struct SmbiosTag {
    header: TagHeader,
    major: u8,
    minor: u8,
    _reserved: [u8; 6],
    tables: [u8],
}

impl SmbiosTag {
    /// Constructs a new tag.
    #[cfg(feature = "builder")]
    #[must_use]
    pub fn new(major: u8, minor: u8, tables: &[u8]) -> Box<Self> {
        let header = TagHeader::new(Self::ID, 0);
        let reserved = [0, 0, 0, 0, 0, 0];
        new_boxed(header, &[&[major, minor], &reserved, tables])
    }

    /// Returns the major number.
    #[must_use]
    pub const fn major(&self) -> u8 {
        self.major
    }

    /// Returns the major number.
    #[must_use]
    pub const fn minor(&self) -> u8 {
        self.minor
    }

    /// Returns the raw tables.
    #[must_use]
    pub const fn tables(&self) -> &[u8] {
        &self.tables
    }
}

impl MaybeDynSized for SmbiosTag {
    type Header = TagHeader;

    const BASE_SIZE: usize = mem::size_of::<TagHeader>() + mem::size_of::<u8>() * 8;

    fn dst_len(header: &TagHeader) -> usize {
        assert!(header.size as usize >= Self::BASE_SIZE);
        header.size as usize - Self::BASE_SIZE
    }
}

impl Tag for SmbiosTag {
    type IDType = TagType;

    const ID: TagType = TagType::Smbios;
}

impl Debug for SmbiosTag {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("BootLoaderNameTag")
            .field("typ", &self.header.typ)
            .field("size", &self.header.size)
            .field("major", &self.major)
            .field("minor", &self.minor)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GenericInfoTag;
    use core::borrow::Borrow;
    use multiboot2_common::test_utils::AlignedBytes;

    #[rustfmt::skip]
    fn get_bytes() -> AlignedBytes<32> {
        AlignedBytes::new([
            TagType::Smbios.val() as u8, 0, 0, 0,
            25, 0, 0, 0,
            /* major */
            7,
            /* minor */
            42,
            /* reserved */
            0, 0, 0, 0, 0, 0,
            /* table data */
            0, 1, 2, 3, 4, 5, 6, 7, 8,
            /* padding */
            0, 0, 0, 0, 0, 0, 0
        ])
    }

    /// Test to parse a given tag.
    #[test]
    fn test_parse() {
        let bytes = get_bytes();
        let tag = GenericInfoTag::ref_from_slice(bytes.borrow()).unwrap();
        let tag = tag.cast::<SmbiosTag>();
        assert_eq!(tag.header.typ, TagType::Smbios);
        assert_eq!(tag.major, 7);
        assert_eq!(tag.minor, 42);
        assert_eq!(&tag.tables, [0, 1, 2, 3, 4, 5, 6, 7, 8]);
    }

    /// Test to generate a tag.
    #[test]
    #[cfg(feature = "builder")]
    fn test_build() {
        let tag = SmbiosTag::new(7, 42, &[0, 1, 2, 3, 4, 5, 6, 7, 8]);
        let bytes = tag.as_bytes().as_ref();
        let bytes = &bytes[..tag.header.size as usize];
        assert_eq!(bytes, &get_bytes()[..tag.header.size as usize]);
    }
}
