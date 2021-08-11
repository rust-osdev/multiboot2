use crate::{HeaderTagFlag, HeaderTagType, StructAsBytes};
use core::mem::size_of;

/// This tag indicates that payload supports starting without terminating UEFI boot services.
/// Or in other words: The payload wants to use UEFI boot services.
#[derive(Copy, Clone, Debug)]
#[repr(C, packed(8))]
pub struct EfiBootServiceHeaderTag {
    typ: HeaderTagType,
    flags: HeaderTagFlag,
    size: u32,
}

impl EfiBootServiceHeaderTag {
    pub const fn new(flags: HeaderTagFlag) -> Self {
        EfiBootServiceHeaderTag {
            typ: HeaderTagType::EfiBS,
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

impl StructAsBytes for EfiBootServiceHeaderTag {}
