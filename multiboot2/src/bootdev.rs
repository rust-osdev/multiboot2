//! Module for [`BootdevTag`].

use crate::{TagHeader, TagType};
use core::mem;
use multiboot2_common::{MaybeDynSized, Tag};

/// The end tag ends the information struct.
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
            header: TagHeader::new(TagType::Apm, mem::size_of::<Self>() as u32),
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

impl MaybeDynSized for BootdevTag {
    type Header = TagHeader;

    const BASE_SIZE: usize = mem::size_of::<Self>();

    fn dst_len(_: &TagHeader) {}
}

impl Tag for BootdevTag {
    type IDType = TagType;

    const ID: TagType = TagType::Bootdev;
}
