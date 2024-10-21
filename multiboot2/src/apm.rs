//! Module for [`ApmTag`].

use crate::{TagHeader, TagType};
use core::mem;
use multiboot2_common::{MaybeDynSized, Tag};

/// The Advanced Power Management (APM) tag.
#[derive(Debug)]
#[repr(C, align(8))]
pub struct ApmTag {
    header: TagHeader,
    version: u16,
    cseg: u16,
    offset: u32,
    cset_16: u16,
    dseg: u16,
    flags: u16,
    cseg_len: u16,
    cseg_16_len: u16,
    dseg_len: u16,
}

impl ApmTag {
    /// Creates a new tag.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        version: u16,
        cseg: u16,
        offset: u32,
        cset_16: u16,
        dset: u16,
        flags: u16,
        cseg_len: u16,
        cseg_16_len: u16,
        dseg_len: u16,
    ) -> Self {
        Self {
            header: TagHeader::new(Self::ID, mem::size_of::<Self>() as u32),
            version,
            cseg,
            offset,
            cset_16,
            dseg: dset,
            flags,
            cseg_len,
            cseg_16_len,
            dseg_len,
        }
    }

    /// The version number of the APM BIOS.
    #[must_use]
    pub const fn version(&self) -> u16 {
        self.version
    }

    /// Contains the 16-bit code segment (CS) address for the APM entry point.
    #[must_use]
    pub const fn cseg(&self) -> u16 {
        self.cseg
    }

    /// Represents the offset address within the code segment (`cseg`) for the
    /// APM entry point.
    #[must_use]
    pub const fn offset(&self) -> u32 {
        self.offset
    }

    /// Contains the 16-bit code segment (CS) address used for 16-bit protected
    /// mode APM functions.
    #[must_use]
    pub const fn cset_16(&self) -> u16 {
        self.cset_16
    }

    /// Holds the 16-bit data segment (DS) address used by the APM BIOS for
    /// data operations.
    #[must_use]
    pub const fn dseg(&self) -> u16 {
        self.dseg
    }

    /// Indicates the status and characteristics of the APM connection, such as
    /// if APM is present and its capabilities.
    #[must_use]
    pub const fn flags(&self) -> u16 {
        self.flags
    }

    /// Indicates the length, in bytes, of the data segment (`dseg`) used by
    /// the APM BIOS
    #[must_use]
    pub const fn cseg_len(&self) -> u16 {
        self.cseg_len
    }

    /// Provides the length, in bytes, of the 16-bit code segment (`cseg_16`)
    /// used for APM functions.
    #[must_use]
    pub const fn cseg_16_len(&self) -> u16 {
        self.cseg_16_len
    }

    /// Indicates the length, in bytes, of the data segment (`dseg`) used by
    /// the APM BIOS.
    #[must_use]
    pub const fn dseg_len(&self) -> u16 {
        self.dseg_len
    }
}

impl MaybeDynSized for ApmTag {
    type Header = TagHeader;

    const BASE_SIZE: usize = mem::size_of::<Self>();

    fn dst_len(_: &TagHeader) {}
}

impl Tag for ApmTag {
    type IDType = TagType;

    const ID: TagType = TagType::Apm;
}
