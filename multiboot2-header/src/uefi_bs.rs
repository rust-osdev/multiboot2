use crate::{HeaderTagFlag, HeaderTagHeader, HeaderTagType};
use core::mem;
use multiboot2_common::{MaybeDynSized, Tag};

/// This tag indicates that payload supports starting without terminating UEFI boot services.
/// Or in other words: The payload wants to use UEFI boot services.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C, align(8))]
pub struct EfiBootServiceHeaderTag {
    header: HeaderTagHeader,
}

impl EfiBootServiceHeaderTag {
    /// Constructs a new tag.
    #[must_use]
    pub const fn new(flags: HeaderTagFlag) -> Self {
        let header =
            HeaderTagHeader::new(HeaderTagType::EfiBS, flags, mem::size_of::<Self>() as u32);
        Self { header }
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
}

impl MaybeDynSized for EfiBootServiceHeaderTag {
    type Header = HeaderTagHeader;

    const BASE_SIZE: usize = mem::size_of::<Self>();

    fn dst_len(_header: &Self::Header) -> Self::Metadata {}
}

impl Tag for EfiBootServiceHeaderTag {
    type IDType = HeaderTagType;
    const ID: HeaderTagType = HeaderTagType::EfiBS;
}

#[cfg(test)]
mod tests {
    use crate::EfiBootServiceHeaderTag;

    #[test]
    fn test_assert_size() {
        assert_eq!(core::mem::size_of::<EfiBootServiceHeaderTag>(), 2 + 2 + 4);
    }
}
