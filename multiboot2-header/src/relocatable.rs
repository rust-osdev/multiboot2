use crate::{HeaderTagFlag, HeaderTagHeader, HeaderTagType};
use core::fmt;
use core::fmt::{Debug, Formatter};
use multiboot2_common::{MaybeDynSized, Tag};

/// Specifies the boot loader's preferred placement for a relocatable image.
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RelocatableHeaderTagPreference {
    /// Let the boot loader choose the image location.
    None = 0,
    /// Load the image at the lowest possible address that is not below
    /// `min_addr`.
    Low = 1,
    /// Load the image at the highest possible address that does not end above
    /// `max_addr`.
    High = 2,
}

/// This tag indicates that the image is relocatable.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C, align(8))]
pub struct RelocatableHeaderTag {
    header: HeaderTagHeader,
    /// Lowest physical address at which the image may be loaded.
    ///
    /// The boot loader cannot load any part of the image below this address.
    min_addr: u32,
    /// Highest physical address at which the loaded image may end.
    ///
    /// The boot loader cannot load any part of the image above this address.
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
        let header =
            HeaderTagHeader::new(HeaderTagType::Relocatable, flags, size_of::<Self>() as u32);
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
            .field("min_addr", &self.min_addr)
            .field("max_addr", &self.max_addr)
            .field("align", &self.align)
            .field("preference", &self.preference)
            .finish()
    }
}

impl MaybeDynSized for RelocatableHeaderTag {
    type Header = HeaderTagHeader;

    const BASE_SIZE: usize = size_of::<Self>();
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
        assert_eq!(size_of::<RelocatableHeaderTag>(), 2 + 2 + 4 + 4 + 4 + 4 + 4);
    }
}
