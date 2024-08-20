use crate::{HeaderTagFlag, HeaderTagHeader, HeaderTagType};
use core::fmt;
use core::fmt::{Debug, Formatter};
use core::mem;
use multiboot2_common::{MaybeDynSized, Tag};

/// This tag is taken into account only on EFI amd64 platforms when Multiboot2 image header
/// contains EFI boot services tag. Then entry point specified in ELF header and the entry address
/// tag of Multiboot2 header are ignored.
///
/// Technically, this is equivalent to the [`crate::EntryAddressHeaderTag`] but with a different
/// [`crate::HeaderTagType`].
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C, align(8))]
pub struct EntryEfi64HeaderTag {
    header: HeaderTagHeader,
    entry_addr: u32,
}

impl EntryEfi64HeaderTag {
    /// Constructs a new tag.
    #[must_use]
    pub const fn new(flags: HeaderTagFlag, entry_addr: u32) -> Self {
        let header = HeaderTagHeader::new(
            HeaderTagType::EntryAddressEFI64,
            flags,
            Self::BASE_SIZE as u32,
        );
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

impl Debug for EntryEfi64HeaderTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("EntryEfi64HeaderTag")
            .field("type", &self.typ())
            .field("flags", &self.flags())
            .field("size", &self.size())
            .field("entry_addr", &(self.entry_addr as *const u32))
            .finish()
    }
}

impl MaybeDynSized for EntryEfi64HeaderTag {
    type Header = HeaderTagHeader;

    const BASE_SIZE: usize = mem::size_of::<HeaderTagHeader>() + mem::size_of::<u32>();

    fn dst_len(_header: &Self::Header) -> Self::Metadata {}
}

impl Tag for EntryEfi64HeaderTag {
    type IDType = HeaderTagType;
    const ID: HeaderTagType = HeaderTagType::EntryAddressEFI64;
}
