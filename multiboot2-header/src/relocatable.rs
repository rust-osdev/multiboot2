use crate::{HeaderTagFlag, HeaderTagHeader, HeaderTagType};
use core::fmt;
use core::fmt::{Debug, Formatter};
use core::mem;
use multiboot2_common::{MaybeDynSized, Tag};

/// It contains load address placement suggestion for boot loader. Boot loader
/// should follow it. ‘0’ means none, ‘1’ means load image at lowest possible address
/// but not lower than min addr and ‘2’ means load image at highest possible
/// address but not higher than max addr.
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RelocatableHeaderTagPreference {
    /// Let boot loader decide.
    None = 0,
    /// Locate at lower end of possible address space.
    Low = 1,
    /// Locate at higher end of possible address space.
    High = 2,
}

/// This tag indicates that the image is relocatable.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C, align(8))]
pub struct RelocatableHeaderTag {
    header: HeaderTagHeader,
    /// Lowest possible physical address at which image should be loaded. The bootloader cannot load any part of image below this address
    min_addr: u32,
    /// Highest possible physical address at which loaded image should end. The bootloader cannot load any part of image above this address.
    max_addr: u32,
    /// Image alignment in memory, e.g. 4096.
    align: u32,
    preference: RelocatableHeaderTagPreference,
}

impl RelocatableHeaderTag {
    /// Constructs a new tag.
    #[must_use]
    pub const fn new(
        flags: HeaderTagFlag,
        min_addr: u32,
        max_addr: u32,
        align: u32,
        preference: RelocatableHeaderTagPreference,
    ) -> Self {
        let header = HeaderTagHeader::new(
            HeaderTagType::Relocatable,
            flags,
            mem::size_of::<Self>() as u32,
        );
        Self {
            header,
            min_addr,
            max_addr,
            align,
            preference,
        }
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

    /// Return the minimum address.
    #[must_use]
    pub const fn min_addr(&self) -> u32 {
        self.min_addr
    }

    /// Return the maximum address.
    #[must_use]
    pub const fn max_addr(&self) -> u32 {
        self.max_addr
    }

    /// Return the alignment.
    #[must_use]
    pub const fn align(&self) -> u32 {
        self.align
    }

    /// Return the preference.
    #[must_use]
    pub const fn preference(&self) -> RelocatableHeaderTagPreference {
        self.preference
    }
}

impl Debug for RelocatableHeaderTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("RelocatableHeaderTag")
            .field("type", &self.typ())
            .field("flags", &self.flags())
            .field("size", &self.size())
            // trick to print this as hexadecimal pointer
            .field("min_addr", &(self.min_addr as *const u32))
            .field("max_addr", &(self.max_addr as *const u32))
            .field("align", &{ self.align })
            .field("preference", &{ self.preference })
            .finish()
    }
}

impl MaybeDynSized for RelocatableHeaderTag {
    type Header = HeaderTagHeader;

    const BASE_SIZE: usize = mem::size_of::<Self>();

    fn dst_len(_header: &Self::Header) -> Self::Metadata {}
}

impl Tag for RelocatableHeaderTag {
    type IDType = HeaderTagType;
    const ID: HeaderTagType = HeaderTagType::Relocatable;
}

#[cfg(test)]
mod tests {
    use crate::RelocatableHeaderTag;

    #[test]
    fn test_assert_size() {
        assert_eq!(
            core::mem::size_of::<RelocatableHeaderTag>(),
            2 + 2 + 4 + 4 + 4 + 4 + 4
        );
    }
}
