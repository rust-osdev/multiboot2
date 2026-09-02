use crate::{HeaderTagFlag, HeaderTagHeader, HeaderTagType};
use multiboot2_common::{MaybeDynSized, Tag};

/// Terminates a list of optional tags in a Multiboot2 header.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C, align(8))]
pub struct EndHeaderTag {
    header: HeaderTagHeader,
}

const _: () = assert!(size_of::<EndHeaderTag>() == 2 + 2 + 4);

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
            HeaderTagType::End,
            HeaderTagFlag::Required,
            size_of::<Self>() as u32,
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

// SAFETY: The tag is repr(C) with the header as first field, any bit
// pattern is valid, and `BASE_SIZE`/`dst_len` match the ABI.
unsafe impl MaybeDynSized for EndHeaderTag {
    type Header = HeaderTagHeader;

    const BASE_SIZE: usize = size_of::<Self>();
}

impl Tag for EndHeaderTag {
    type IDType = HeaderTagType;
    const ID: HeaderTagType = HeaderTagType::End;
}
