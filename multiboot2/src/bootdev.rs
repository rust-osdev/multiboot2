//! Module for [`BootdevTag`].

use crate::{TagHeader, TagType};
use multiboot2_common::{MaybeDynSized, Tag};

/// Identifies the BIOS device and partition that supplied the OS image.
#[derive(Debug)]
#[repr(C, align(8))]
pub struct BootdevTag {
    header: TagHeader,
    biosdev: u32,
    slice: u32,
    part: u32,
}

impl BootdevTag {
    /// Creates a new tag.
    #[must_use]
    pub fn new(biosdev: u32, slice: u32, part: u32) -> Self {
        Self {
            header: TagHeader::new(Self::ID, Self::BASE_SIZE as u32),
            biosdev,
            slice,
            part,
        }
    }

    /// Returns the bios device from which the device was booted from.
    /// `0x00` represents the first floppy disk.
    /// `0x80` represents the first hard disk, 0x81 the second hard disk, and
    /// so on.
    #[must_use]
    pub const fn biosdev(&self) -> u32 {
        self.biosdev
    }

    /// The slice field identifies the partition (also known as a "slice" in BSD
    /// terminology) on the BIOS device from which the operating system was
    /// booted.
    #[must_use]
    pub const fn slice(&self) -> u32 {
        self.slice
    }

    /// The part field denotes the subpartition or logical partition within the
    /// primary partition (if applicable) from which the operating system was
    /// booted.
    #[must_use]
    pub const fn part(&self) -> u32 {
        self.part
    }
}

// SAFETY: The tag is repr(C) with the header as first field, any bit
// pattern is valid, and `BASE_SIZE`/`dst_len` match the ABI.
unsafe impl MaybeDynSized for BootdevTag {
    type Header = TagHeader;

    // Spec size (20), excluding the trailing padding that `size_of::<Self>()`
    // (24) would add.
    const BASE_SIZE: usize = size_of::<TagHeader>() + 3 * size_of::<u32>();
}

impl Tag for BootdevTag {
    type IDType = TagType;

    const ID: TagType = TagType::Bootdev;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reported_size_matches_spec() {
        // The Multiboot2 spec mandates size 20 (excluding padding), not the
        // 24 bytes that `size_of::<BootdevTag>()` would report.
        let tag = BootdevTag::new(0x80, 0, 0xffff_ffff);
        assert_eq!(tag.header.size, 20);
    }
}
