use crate::{HeaderTagFlag, HeaderTagHeader, HeaderTagType};
use core::mem::size_of;

/// Possible flags for [`ConsoleHeaderTag`].
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConsoleHeaderTagFlags {
    /// Console required.
    ConsoleRequired = 0,
    /// EGA text support.
    EgaTextSupported = 1,
}

/// Tells that a console must be available in MBI.
/// Only relevant for legacy BIOS.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct ConsoleHeaderTag {
    header: HeaderTagHeader,
    console_flags: ConsoleHeaderTagFlags,
}

impl ConsoleHeaderTag {
    /// Constructs a new tag.
    #[must_use]
    pub const fn new(flags: HeaderTagFlag, console_flags: ConsoleHeaderTagFlags) -> Self {
        let header =
            HeaderTagHeader::new(HeaderTagType::ConsoleFlags, flags, size_of::<Self>() as u32);
        Self {
            header,
            console_flags,
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

    /// Returns the [`ConsoleHeaderTagFlags`].
    #[must_use]
    pub const fn console_flags(&self) -> ConsoleHeaderTagFlags {
        self.console_flags
    }
}

#[cfg(test)]
mod tests {
    use crate::ConsoleHeaderTag;

    #[test]
    fn test_assert_size() {
        assert_eq!(core::mem::size_of::<ConsoleHeaderTag>(), 2 + 2 + 4 + 4);
    }
}
