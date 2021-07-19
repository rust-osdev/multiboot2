use alloc::fmt;
use core::fmt::{Debug, Formatter};
use core::mem::size_of;
use {HeaderTagFlag, HeaderTagType};

/// Specifies the physical address to which the boot loader should jump in
/// order to start running the operating system.
/// Not needed for ELF files.
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct EntryHeaderTag {
    typ: HeaderTagType,
    flags: HeaderTagFlag,
    size: u32,
    entry_addr: u32,
}

impl EntryHeaderTag {
    pub fn new(flags: HeaderTagFlag, entry_addr: u32) -> Self {
        EntryHeaderTag {
            typ: HeaderTagType::EntryAddress,
            flags,
            size: size_of::<Self>() as u32,
            entry_addr,
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
    pub fn entry_addr(&self) -> u32 {
        self.entry_addr
    }
}

impl Debug for EntryHeaderTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("EntryHeaderTag")
            .field("type", &{ self.typ })
            .field("flags", &{ self.flags })
            .field("size", &{ self.size })
            .field("entry_addr", &(self.entry_addr as *const u32))
            .finish()
    }
}
