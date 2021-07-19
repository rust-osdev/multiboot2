use core::mem::size_of;
use {HeaderTagFlag, HeaderTagType};

/// Specifies the preferred graphics mode. If this tag
/// is present the bootloader assumes that the payload
/// has framebuffer support. Note: This is only a
/// recommended mode. Only relevant on legacy BIOS.
#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct FramebufferHeaderTag {
    typ: HeaderTagType,
    flags: HeaderTagFlag,
    size: u32,
    width: u32,
    height: u32,
    depth: u32,
}

impl FramebufferHeaderTag {
    pub fn new(flags: HeaderTagFlag, width: u32, height: u32, depth: u32) -> Self {
        FramebufferHeaderTag {
            typ: HeaderTagType::Framebuffer,
            flags,
            size: size_of::<Self>() as u32,
            width,
            height,
            depth,
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
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
    pub fn depth(&self) -> u32 {
        self.depth
    }
}
