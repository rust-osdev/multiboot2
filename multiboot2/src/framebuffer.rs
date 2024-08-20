//! Module for [`FramebufferTag`].

use crate::tag::TagHeader;
use crate::TagType;
use core::fmt::Debug;
use core::mem;
use core::slice;
use derive_more::Display;
use multiboot2_common::{MaybeDynSized, Tag};
#[cfg(feature = "builder")]
use {alloc::boxed::Box, multiboot2_common::new_boxed};

/// TODO this memory reader is unsafe and causes UB, according to Miri.
/// We need to replace it.
///
/// Helper struct to read bytes from a raw pointer and increase the pointer
/// automatically.
struct Reader {
    ptr: *const u8,
    off: usize,
}

impl Reader {
    const fn new<T>(ptr: *const T) -> Self {
        Self {
            ptr: ptr as *const u8,
            off: 0,
        }
    }

    fn read_u8(&mut self) -> u8 {
        self.off += 1;
        unsafe { *self.ptr.add(self.off - 1) }
    }

    fn read_u16(&mut self) -> u16 {
        self.read_u8() as u16 | (self.read_u8() as u16) << 8
    }

    fn read_u32(&mut self) -> u32 {
        self.read_u16() as u32 | (self.read_u16() as u32) << 16
    }

    fn current_address(&self) -> usize {
        unsafe { self.ptr.add(self.off) as usize }
    }
}

/// The VBE Framebuffer information tag.
#[derive(ptr_meta::Pointee, Eq)]
#[repr(C, align(8))]
pub struct FramebufferTag {
    header: TagHeader,

    /// Contains framebuffer physical address.
    ///
    /// This field is 64-bit wide but bootloader should set it under 4GiB if
    /// possible for compatibility with payloads which aren’t aware of PAE or
    /// amd64.
    address: u64,

    /// Contains the pitch in bytes.
    pitch: u32,

    /// Contains framebuffer width in pixels.
    width: u32,

    /// Contains framebuffer height in pixels.
    height: u32,

    /// Contains number of bits per pixel.
    bpp: u8,

    /// The type of framebuffer, one of: `Indexed`, `RGB` or `Text`.
    type_no: u8,

    // In the multiboot spec, it has this listed as a u8 _NOT_ a u16.
    // Reading the GRUB2 source code reveals it is in fact a u16.
    _reserved: u16,

    // TODO This situation is currently not properly typed. Someone needs to
    // look into it.
    buffer: [u8],
}

impl FramebufferTag {
    /// Constructs a new tag.
    #[cfg(feature = "builder")]
    #[must_use]
    pub fn new(
        address: u64,
        pitch: u32,
        width: u32,
        height: u32,
        bpp: u8,
        buffer_type: FramebufferTypeId,
    ) -> Box<Self> {
        let header = TagHeader::new(Self::ID, 0);
        let address = address.to_ne_bytes();
        let pitch = pitch.to_ne_bytes();
        let width = width.to_ne_bytes();
        let height = height.to_ne_bytes();
        let padding = [0; 2];
        new_boxed(
            header,
            &[
                &address,
                &pitch,
                &width,
                &height,
                &[bpp],
                &[buffer_type as u8],
                &padding,
            ],
        )
    }

    /// Contains framebuffer physical address.
    ///
    /// This field is 64-bit wide but bootloader should set it under 4GiB if
    /// possible for compatibility with payloads which aren’t aware of PAE or
    /// amd64.
    #[must_use]
    pub const fn address(&self) -> u64 {
        self.address
    }

    /// Contains the pitch in bytes.
    #[must_use]
    pub const fn pitch(&self) -> u32 {
        self.pitch
    }

    /// Contains framebuffer width in pixels.
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Contains framebuffer height in pixels.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// Contains number of bits per pixel.
    #[must_use]
    pub const fn bpp(&self) -> u8 {
        self.bpp
    }

    /// TODO unsafe. Someone needs to fix this. This causes UB according to Miri.
    ///  Dont forget to reenable all test usages once fixed.
    ///
    /// The type of framebuffer, one of: `Indexed`, `RGB` or `Text`.
    ///
    /// # Safety
    /// This function needs refactoring. This was never safe, since the beginning.
    pub unsafe fn buffer_type(&self) -> Result<FramebufferType, UnknownFramebufferType> {
        let mut reader = Reader::new(self.buffer.as_ptr());
        let typ = FramebufferTypeId::try_from(self.type_no)?;
        match typ {
            FramebufferTypeId::Indexed => {
                let num_colors = reader.read_u32();
                // TODO static cast looks like UB?
                let palette = unsafe {
                    slice::from_raw_parts(
                        reader.current_address() as *const FramebufferColor,
                        num_colors as usize,
                    )
                } as &'static [FramebufferColor];
                Ok(FramebufferType::Indexed { palette })
            }
            FramebufferTypeId::RGB => {
                let red_pos = reader.read_u8(); // These refer to the bit positions of the LSB of each field
                let red_mask = reader.read_u8(); // And then the length of the field from LSB to MSB
                let green_pos = reader.read_u8();
                let green_mask = reader.read_u8();
                let blue_pos = reader.read_u8();
                let blue_mask = reader.read_u8();
                Ok(FramebufferType::RGB {
                    red: FramebufferField {
                        position: red_pos,
                        size: red_mask,
                    },
                    green: FramebufferField {
                        position: green_pos,
                        size: green_mask,
                    },
                    blue: FramebufferField {
                        position: blue_pos,
                        size: blue_mask,
                    },
                })
            }
            FramebufferTypeId::Text => Ok(FramebufferType::Text),
        }
    }
}

impl MaybeDynSized for FramebufferTag {
    type Header = TagHeader;

    const BASE_SIZE: usize = mem::size_of::<TagHeader>()
        + mem::size_of::<u64>()
        + 3 * mem::size_of::<u32>()
        + 2 * mem::size_of::<u8>()
        + mem::size_of::<u16>();

    fn dst_len(header: &TagHeader) -> usize {
        assert!(header.size as usize >= Self::BASE_SIZE);
        header.size as usize - Self::BASE_SIZE
    }
}

impl Tag for FramebufferTag {
    type IDType = TagType;

    const ID: TagType = TagType::Framebuffer;
}

impl Debug for FramebufferTag {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FramebufferTag")
            .field("typ", &self.header.typ)
            .field("size", &self.header.size)
            // TODO unsafe. Fix in a follow-up commit
            //.field("buffer_type", &self.buffer_type())
            .field("address", &self.address)
            .field("pitch", &self.pitch)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("bpp", &self.bpp)
            .finish()
    }
}

impl PartialEq for FramebufferTag {
    fn eq(&self, other: &Self) -> bool {
        self.header == other.header
            && self.address == { other.address }
            && self.pitch == { other.pitch }
            && self.width == { other.width }
            && self.height == { other.height }
            && self.bpp == { other.bpp }
            && self.type_no == { other.type_no }
            && self.buffer == other.buffer
    }
}

/// Helper struct for [`FramebufferType`].
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
#[allow(clippy::upper_case_acronyms)]
pub enum FramebufferTypeId {
    Indexed = 0,
    RGB = 1,
    Text = 2,
    // spec says: there may be more variants in the future
}

impl TryFrom<u8> for FramebufferTypeId {
    type Error = UnknownFramebufferType;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Indexed),
            1 => Ok(Self::RGB),
            2 => Ok(Self::Text),
            val => Err(UnknownFramebufferType(val)),
        }
    }
}

impl From<FramebufferType<'_>> for FramebufferTypeId {
    fn from(value: FramebufferType) -> Self {
        match value {
            FramebufferType::Indexed { .. } => Self::Indexed,
            FramebufferType::RGB { .. } => Self::RGB,
            FramebufferType::Text => Self::Text,
        }
    }
}

/// The type of framebuffer.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FramebufferType<'a> {
    /// Indexed color.
    Indexed {
        #[allow(missing_docs)]
        palette: &'a [FramebufferColor],
    },

    /// Direct RGB color.
    #[allow(missing_docs)]
    #[allow(clippy::upper_case_acronyms)]
    RGB {
        red: FramebufferField,
        green: FramebufferField,
        blue: FramebufferField,
    },

    /// EGA Text.
    ///
    /// In this case the framebuffer width and height are expressed in
    /// characters and not in pixels.
    ///
    /// The bpp is equal 16 (16 bits per character) and pitch is expressed in bytes per text line.
    Text,
}

/// An RGB color type field.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct FramebufferField {
    /// Color field position.
    pub position: u8,

    /// Color mask size.
    pub size: u8,
}

/// A framebuffer color descriptor in the palette.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)] // no align(8) here is correct
pub struct FramebufferColor {
    /// The Red component of the color.
    pub red: u8,

    /// The Green component of the color.
    pub green: u8,

    /// The Blue component of the color.
    pub blue: u8,
}

/// Error when an unknown [`FramebufferTypeId`] is found.
#[derive(Debug, Copy, Clone, Display, PartialEq, Eq)]
#[display(fmt = "Unknown framebuffer type {}", _0)]
pub struct UnknownFramebufferType(u8);

#[cfg(feature = "unstable")]
impl core::error::Error for UnknownFramebufferType {}

#[cfg(test)]
mod tests {
    use super::*;

    // Compile time test
    #[test]
    fn test_size() {
        assert_eq!(mem::size_of::<FramebufferColor>(), 3)
    }

    #[test]
    #[cfg(feature = "builder")]
    fn create_new() {
        let tag = FramebufferTag::new(0x1000, 1, 1024, 1024, 8, FramebufferTypeId::Indexed);
        dbg!(tag);
    }
}
