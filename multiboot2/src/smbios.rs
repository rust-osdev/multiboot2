//! Module for [`SmbiosTag`].

#[cfg(feature = "builder")]
use crate::builder::BoxedDst;
use crate::tag::TagHeader;
use crate::{TagTrait, TagType};
use core::fmt::Debug;
use core::mem;

const METADATA_SIZE: usize = mem::size_of::<TagHeader>() + mem::size_of::<u8>() * 8;

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
    pub fn new(major: u8, minor: u8, tables: &[u8]) -> BoxedDst<Self> {
        let mut bytes = [major, minor, 0, 0, 0, 0, 0, 0].to_vec();
        bytes.extend(tables);
        BoxedDst::new(&bytes)
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

impl TagTrait for SmbiosTag {
    const ID: TagType = TagType::Smbios;

    fn dst_len(header: &TagHeader) -> usize {
        assert!(header.size as usize >= METADATA_SIZE);
        header.size as usize - METADATA_SIZE
    }
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
    use crate::tag::{GenericTag, TagBytesRef};
    use crate::test_util::AlignedBytes;

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
        let bytes = TagBytesRef::try_from(&bytes[..]).unwrap();
        let tag = GenericTag::ref_from(bytes);
        let tag = tag.cast::<SmbiosTag>();
        assert_eq!(tag.header.typ, TagType::Smbios);
        assert_eq!(tag.major, 7);
        assert_eq!(tag.minor, 42);
        assert_eq!(&tag.tables, [0, 1, 2, 3, 4, 5, 6, 7, 8]);
    }

    /// Test to generate a tag.
    #[test]
    #[cfg(feature = "builder")]
    #[ignore]
    fn test_build() {
        let tag = SmbiosTag::new(7, 42, &[0, 1, 2, 3, 4, 5, 6, 7, 8]);
        let bytes = tag.as_bytes();
        assert_eq!(bytes, &get_bytes()[..]);
    }
}
