use crate::{HeaderTagFlag, HeaderTagHeader, HeaderTagType};
use core::mem;

/// Specifies the preferred graphics mode. If this tag
/// is present the bootloader assumes that the payload
/// has framebuffer support. Note: This is only a
/// recommended mode. Only relevant on legacy BIOS.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct FramebufferHeaderTag {
    header: HeaderTagHeader,
    width: u32,
    height: u32,
    depth: u32,
}

impl FramebufferHeaderTag {
    /// Constructs a new tag.
    #[must_use]
    pub const fn new(flags: HeaderTagFlag, width: u32, height: u32, depth: u32) -> Self {
        let header = HeaderTagHeader::new(
            HeaderTagType::Framebuffer,
            flags,
            mem::size_of::<Self>() as u32,
        );
        Self {
            header,
            width,
            height,
            depth,
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

    /// Returns the width.
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Returns the height.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// Returns the depth.
    #[must_use]
    pub const fn depth(&self) -> u32 {
        self.depth
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assert_size() {
        assert_eq!(
            mem::size_of::<FramebufferHeaderTag>(),
            2 + 2 + 4 + 4 + 4 + 4
        );
    }
}
