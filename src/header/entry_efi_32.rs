use alloc::fmt;
use core::fmt::{Debug, Formatter};
use core::mem::size_of;
use {HeaderTagFlag, HeaderTagType};

/// This tag is taken into account only on EFI i386 platforms when Multiboot2 image header
/// contains EFI boot services tag. Then entry point specified in ELF header and the entry address tag of Multiboot2 header are ignored.
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct EntryEfi32HeaderTag {
    typ: HeaderTagType,
    flags: HeaderTagFlag,
    size: u32,
    entry_addr: u32,
}

impl EntryEfi32HeaderTag {
    pub fn new(flags: HeaderTagFlag, entry_addr: u32) -> Self {
        EntryEfi32HeaderTag {
            typ: HeaderTagType::EntryAddressEFI32,
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

impl Debug for EntryEfi32HeaderTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("EntryEfi32HeaderTag")
            .field("type", &{ self.typ })
            .field("flags", &{ self.flags })
            .field("size", &{ self.size })
            .field("entry_addr", &(self.entry_addr as *const u32))
            .finish()
    }
}
