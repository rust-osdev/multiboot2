use crate::{HeaderTagFlag, HeaderTagType, StructAsBytes};
use core::mem::size_of;

/// Specifies the preferred graphics mode. If this tag
/// is present the bootloader assumes that the payload
/// has framebuffer support. Note: This is only a
/// recommended mode. Only relevant on legacy BIOS.
#[derive(Copy, Clone, Debug)]
#[repr(C, packed(8))]
pub struct FramebufferHeaderTag {
    typ: HeaderTagType,
    flags: HeaderTagFlag,
    size: u32,
    width: u32,
    height: u32,
    depth: u32,
}

impl FramebufferHeaderTag {
    pub const fn new(flags: HeaderTagFlag, width: u32, height: u32, depth: u32) -> Self {
        FramebufferHeaderTag {
            typ: HeaderTagType::Framebuffer,
            flags,
            size: size_of::<Self>() as u32,
            width,
            height,
            depth,
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
    pub const fn width(&self) -> u32 {
        self.width
    }
    pub const fn height(&self) -> u32 {
        self.height
    }
    pub const fn depth(&self) -> u32 {
        self.depth
    }
}

impl StructAsBytes for FramebufferHeaderTag {}
