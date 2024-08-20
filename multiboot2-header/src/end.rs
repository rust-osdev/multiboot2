use crate::{HeaderTagFlag, HeaderTagHeader, HeaderTagType};
use core::mem;
use multiboot2_common::{MaybeDynSized, Tag};

/// Terminates a list of optional tags in a Multiboot2 header.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct EndHeaderTag {
    header: HeaderTagHeader,
}

impl Default for EndHeaderTag {
    fn default() -> Self {
        Self::new()
    }
}

impl EndHeaderTag {
    /// Constructs a new tag.
    #[must_use]
    pub const fn new() -> Self {
        let header = HeaderTagHeader::new(
            HeaderTagType::EntryAddress,
            HeaderTagFlag::Required,
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

impl MaybeDynSized for EndHeaderTag {
    type Header = HeaderTagHeader;

    const BASE_SIZE: usize = mem::size_of::<Self>();

    fn dst_len(_header: &Self::Header) -> Self::Metadata {}
}

impl Tag for EndHeaderTag {
    type IDType = HeaderTagType;
    const ID: HeaderTagType = HeaderTagType::End;
}

#[cfg(test)]
mod tests {
    use crate::EndHeaderTag;

    #[test]
    fn test_assert_size() {
        assert_eq!(core::mem::size_of::<EndHeaderTag>(), 2 + 2 + 4);
    }
}
