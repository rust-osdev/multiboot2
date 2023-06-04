use crate::{HeaderTagFlag, HeaderTagType};
use core::fmt;
use core::fmt::{Debug, Formatter};
use core::mem::size_of;

/// It contains load address placement suggestion for boot loader. Boot loader
/// should follow it. ‘0’ means none, ‘1’ means load image at lowest possible address
/// but not lower than min addr and ‘2’ means load image at highest possible
/// address but not higher than max addr.
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum RelocatableHeaderTagPreference {
    /// Let boot loader decide.
    None = 0,
    /// Locate at lower end of possible address space.
    Low = 1,
    /// Locate at higher end of possible address space.
    High = 2,
}

/// This tag indicates that the image is relocatable.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct RelocatableHeaderTag {
    typ: HeaderTagType,
    flags: HeaderTagFlag,
    size: u32,
    /// Lowest possible physical address at which image should be loaded. The bootloader cannot load any part of image below this address
    min_addr: u32,
    /// Highest possible physical address at which loaded image should end. The bootloader cannot load any part of image above this address.
    max_addr: u32,
    /// Image alignment in memory, e.g. 4096.
    align: u32,
    preference: RelocatableHeaderTagPreference,
}

impl RelocatableHeaderTag {
    pub const fn new(
        flags: HeaderTagFlag,
        min_addr: u32,
        max_addr: u32,
        align: u32,
        preference: RelocatableHeaderTagPreference,
    ) -> Self {
        RelocatableHeaderTag {
            typ: HeaderTagType::Relocatable,
            flags,
            size: size_of::<Self>() as u32,
            min_addr,
            max_addr,
            align,
            preference,
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
    pub const fn min_addr(&self) -> u32 {
        self.min_addr
    }
    pub const fn max_addr(&self) -> u32 {
        self.max_addr
    }
    pub const fn align(&self) -> u32 {
        self.align
    }
    pub const fn preference(&self) -> RelocatableHeaderTagPreference {
        self.preference
    }
}

impl Debug for RelocatableHeaderTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("RelocatableHeaderTag")
            .field("type", &{ self.typ })
            .field("flags", &{ self.flags })
            .field("size", &{ self.size })
            // trick to print this as hexadecimal pointer
            .field("min_addr", &(self.min_addr as *const u32))
            .field("max_addr", &(self.max_addr as *const u32))
            .field("align", &{ self.align })
            .field("preference", &{ self.preference })
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::RelocatableHeaderTag;

    #[test]
    fn test_assert_size() {
        assert_eq!(
            core::mem::size_of::<RelocatableHeaderTag>(),
            2 + 2 + 4 + 4 + 4 + 4 + 4
        );
    }
}
