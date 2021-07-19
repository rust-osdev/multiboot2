use alloc::fmt;
use core::fmt::{Debug, Formatter};
use core::mem::size_of;
use {HeaderTagFlag, HeaderTagType};

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
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
#[repr(C, packed)]
pub struct RelocatableHeaderTag {
    typ: HeaderTagType,
    flags: HeaderTagFlag,
    size: u32,
    min_addr: u32,
    max_addr: u32,
    align: u32,
    preference: RelocatableHeaderTagPreference,
}

impl RelocatableHeaderTag {
    pub fn new(
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

    pub fn typ(&self) -> HeaderTagType {
        self.typ
    }
    pub fn flags(&self) -> HeaderTagFlag {
        self.flags
    }
    pub fn size(&self) -> u32 {
        self.size
    }
    pub fn min_addr(&self) -> u32 {
        self.min_addr
    }
    pub fn max_addr(&self) -> u32 {
        self.max_addr
    }
    pub fn align(&self) -> u32 {
        self.align
    }
    pub fn preference(&self) -> RelocatableHeaderTagPreference {
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
