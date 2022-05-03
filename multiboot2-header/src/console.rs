use crate::{HeaderTagFlag, HeaderTagType};
use core::mem::size_of;

/// Possible flags for [`ConsoleHeaderTag`].
#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum ConsoleHeaderTagFlags {
    /// Console required.
    ConsoleRequired = 0,
    /// EGA text support.
    EgaTextSupported = 1,
}

/// Tells that a console must be available in MBI.
/// Only relevant for legacy BIOS.
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct ConsoleHeaderTag {
    typ: HeaderTagType,
    flags: HeaderTagFlag,
    size: u32,
    console_flags: ConsoleHeaderTagFlags,
}

impl ConsoleHeaderTag {
    pub const fn new(flags: HeaderTagFlag, console_flags: ConsoleHeaderTagFlags) -> Self {
        ConsoleHeaderTag {
            typ: HeaderTagType::ConsoleFlags,
            flags,
            size: size_of::<Self>() as u32,
            console_flags,
        }
    }

    pub const fn typ(&self) -> HeaderTagType {
        self.typ
    }
    pub const fn flags(&self) -> HeaderTagFlag {
        self.flags
    }
    pub const fn size(&self) -> u32 {
        self.size
    }
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
