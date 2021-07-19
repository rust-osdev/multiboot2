use core::mem::size_of;
use {HeaderTagFlag, HeaderTagType};

/// This tag indicates that payload supports starting without terminating UEFI boot services.
#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct EfiBootServiceHeaderTag {
    typ: HeaderTagType,
    flags: HeaderTagFlag,
    size: u32,
}

impl EfiBootServiceHeaderTag {
    pub fn new(flags: HeaderTagFlag) -> Self {
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
