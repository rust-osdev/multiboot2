use core::mem::size_of;
use {HeaderTagFlag, HeaderTagType};

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
#[repr(C, packed)]
pub struct ConsoleHeaderTag {
    typ: HeaderTagType,
    flags: HeaderTagFlag,
    size: u32,
    console_flags: ConsoleHeaderTagFlags,
}

impl ConsoleHeaderTag {
    pub fn new(flags: HeaderTagFlag, console_flags: ConsoleHeaderTagFlags) -> Self {
        ConsoleHeaderTag {
            typ: HeaderTagType::ConsoleFlags,
            flags,
            size: size_of::<Self>() as u32,
            console_flags,
        }
    }

    pub fn typ(&self) -> HeaderTagType {
        self.typ
    }
    pub fn flags(&self) -> HeaderTagFlag {
        self.flags
    }
    pub fn size(&self) -> u32 {
        self.size
    }
    pub fn console_flags(&self) -> ConsoleHeaderTagFlags {
        self.console_flags
    }
}
