use crate::{HeaderTagFlag, HeaderTagHeader, HeaderTagType};
use core::fmt;
use core::fmt::{Debug, Formatter};
use core::mem;
use multiboot2_common::{MaybeDynSized, Tag};

/// Specifies the physical address to which the boot loader should jump in
/// order to start running the operating system. Not needed for ELF files.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C, align(8))]
pub struct EntryAddressHeaderTag {
    header: HeaderTagHeader,
    entry_addr: u32,
}

impl EntryAddressHeaderTag {
    /// Constructs a new tag.
    #[must_use]
    pub const fn new(flags: HeaderTagFlag, entry_addr: u32) -> Self {
        let header =
            HeaderTagHeader::new(HeaderTagType::EntryAddress, flags, Self::BASE_SIZE as u32);
        Self { header, entry_addr }
    }

    /// Returns the [`HeaderTagType`].
    #[must_use]
    pub const fn typ(&self) -> HeaderTagType {
        self.header.typ()
    }

    /// Returns the [`HeaderTagFlag`]s.
    #[must_use]
    pub const fn flags(&self) -> HeaderTagFlag {
        self.header.flags()
    }

    /// Returns the size.
    #[must_use]
    pub const fn size(&self) -> u32 {
        self.header.size()
    }

    /// Returns the entry address.
    #[must_use]
    pub const fn entry_addr(&self) -> u32 {
        self.entry_addr
    }
}

impl Debug for EntryAddressHeaderTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("EntryAddressHeaderTag")
            .field("type", &self.typ())
            .field("flags", &self.flags())
            .field("size", &self.size())
            .field("entry_addr", &(self.entry_addr as *const u32))
            .finish()
    }
}

impl MaybeDynSized for EntryAddressHeaderTag {
    type Header = HeaderTagHeader;

    const BASE_SIZE: usize = mem::size_of::<HeaderTagHeader>() + mem::size_of::<u32>();

    fn dst_len(_header: &Self::Header) -> Self::Metadata {}
}

impl Tag for EntryAddressHeaderTag {
    type IDType = HeaderTagType;
    const ID: HeaderTagType = HeaderTagType::EntryAddress;
}
