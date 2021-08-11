use crate::{HeaderTagFlag, HeaderTagType, StructAsBytes};
use core::mem::size_of;

/// If this tag is present, provided boot modules must be page aligned.
#[derive(Copy, Clone, Debug)]
#[repr(C, packed(8))]
pub struct ModuleAlignHeaderTag {
    typ: HeaderTagType,
    flags: HeaderTagFlag,
    size: u32,
}

impl ModuleAlignHeaderTag {
    pub const fn new(flags: HeaderTagFlag) -> Self {
        ModuleAlignHeaderTag {
            typ: HeaderTagType::ModuleAlign,
            flags,
            size: size_of::<Self>() as u32,
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
}

impl StructAsBytes for ModuleAlignHeaderTag {}
