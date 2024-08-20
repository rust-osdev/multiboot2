use crate::{HeaderTagFlag, HeaderTagHeader, HeaderTagType};
use core::mem;
use multiboot2_common::{MaybeDynSized, Tag};

/// If this tag is present, provided boot modules must be page aligned.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C, align(8))]
pub struct ModuleAlignHeaderTag {
    header: HeaderTagHeader,
}

impl ModuleAlignHeaderTag {
    /// Constructs a new tag.
    #[must_use]
    pub const fn new(flags: HeaderTagFlag) -> Self {
        let header = HeaderTagHeader::new(
            HeaderTagType::ModuleAlign,
            flags,
            mem::size_of::<Self>() as u32,
        );
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

impl MaybeDynSized for ModuleAlignHeaderTag {
    type Header = HeaderTagHeader;

    const BASE_SIZE: usize = mem::size_of::<Self>();

    fn dst_len(_header: &Self::Header) -> Self::Metadata {}
}

impl Tag for ModuleAlignHeaderTag {
    type IDType = HeaderTagType;
    const ID: HeaderTagType = HeaderTagType::ModuleAlign;
}

#[cfg(test)]
mod tests {
    use crate::ModuleAlignHeaderTag;

    #[test]
    fn test_assert_size() {
        assert_eq!(core::mem::size_of::<ModuleAlignHeaderTag>(), 2 + 2 + 4);
    }
}
