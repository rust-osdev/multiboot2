use crate::{HeaderTagFlag, HeaderTagType};
use core::fmt;
use core::fmt::{Debug, Formatter};
use core::mem::size_of;

/// Specifies the physical address to which the boot loader should jump in
/// order to start running the operating system. Not needed for ELF files.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct EntryAddressHeaderTag {
    typ: HeaderTagType,
    flags: HeaderTagFlag,
    size: u32,
    entry_addr: u32,
}

impl EntryAddressHeaderTag {
    pub const fn new(flags: HeaderTagFlag, entry_addr: u32) -> Self {
        EntryAddressHeaderTag {
            typ: HeaderTagType::EntryAddress,
            flags,
            size: size_of::<Self>() as u32,
            entry_addr,
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
    pub const fn entry_addr(&self) -> u32 {
        self.entry_addr
    }
}

impl Debug for EntryAddressHeaderTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("EntryAddressHeaderTag")
            .field("type", &{ self.typ })
            .field("flags", &{ self.flags })
            .field("size", &{ self.size })
            .field("entry_addr", &(self.entry_addr as *const u32))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::EntryAddressHeaderTag;

    #[test]
    fn test_assert_size() {
        assert_eq!(core::mem::size_of::<EntryAddressHeaderTag>(), 2 + 2 + 4 + 4);
    }
}
