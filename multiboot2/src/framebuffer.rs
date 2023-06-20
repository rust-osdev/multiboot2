use crate::{Tag, TagTrait, TagTypeId};

use core::fmt::Debug;
use core::mem::size_of;
use core::slice;
use derive_more::Display;

#[cfg(feature = "builder")]
use {
    crate::builder::boxed_dst_tag, crate::builder::traits::StructAsBytes, crate::TagType,
    alloc::boxed::Box, alloc::vec::Vec,
};

/// Helper struct to read bytes from a raw pointer and increase the pointer
/// automatically.
struct Reader {
    ptr: *const u8,
    off: usize,
}

impl Reader {
    fn new<T>(ptr: *const T) -> Reader {
        Reader {
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

const METADATA_SIZE: usize = size_of::<TagTypeId>()
    + 4 * size_of::<u32>()
    + size_of::<u64>()
    + size_of::<u16>()
    + 2 * size_of::<u8>();

/// The VBE Framebuffer information Tag.
#[derive(ptr_meta::Pointee, Eq)]
#[repr(C)]
pub struct FramebufferTag {
    typ: TagTypeId,
    size: u32,

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

    buffer: [u8],
}

impl FramebufferTag {
    #[cfg(feature = "builder")]
    pub fn new(
        address: u64,
        pitch: u32,
        width: u32,
        height: u32,
        bpp: u8,
        buffer_type: FramebufferType,
    ) -> Box<Self> {
        let mut bytes: Vec<u8> = address.to_le_bytes().into();
        bytes.extend(pitch.to_le_bytes());
        bytes.extend(width.to_le_bytes());
        bytes.extend(height.to_le_bytes());
        bytes.extend(bpp.to_le_bytes());
        bytes.extend(buffer_type.to_bytes());
        boxed_dst_tag(TagType::Framebuffer, &bytes)
    }

    /// Contains framebuffer physical address.
    ///
    /// This field is 64-bit wide but bootloader should set it under 4GiB if
    /// possible for compatibility with payloads which aren’t aware of PAE or
    /// amd64.
    pub fn address(&self) -> u64 {
        self.address
    }

    /// Contains the pitch in bytes.
    pub fn pitch(&self) -> u32 {
        self.pitch
    }

    /// Contains framebuffer width in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Contains framebuffer height in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Contains number of bits per pixel.
    pub fn bpp(&self) -> u8 {
        self.bpp
    }

    /// The type of framebuffer, one of: `Indexed`, `RGB` or `Text`.
    pub fn buffer_type(&self) -> Result<FramebufferType, UnknownFramebufferType> {
        let mut reader = Reader::new(self.buffer.as_ptr());
        let typ = FramebufferTypeId::try_from(self.type_no)?;
        match typ {
            FramebufferTypeId::Indexed => {
                let num_colors = reader.read_u32();
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

impl TagTrait for FramebufferTag {
    fn dst_size(base_tag: &Tag) -> usize {
        assert!(base_tag.size as usize >= METADATA_SIZE);
        base_tag.size as usize - METADATA_SIZE
    }
}

#[cfg(feature = "builder")]
impl StructAsBytes for FramebufferTag {
    fn byte_size(&self) -> usize {
        self.size.try_into().unwrap()
    }
}

impl Debug for FramebufferTag {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FramebufferTag")
            .field("typ", &{ self.typ })
            .field("size", &{ self.size })
            .field("buffer_type", &self.buffer_type())
            .field("address", &{ self.address })
            .field("pitch", &{ self.pitch })
            .field("width", &{ self.width })
            .field("height", &{ self.height })
            .field("bpp", &self.bpp)
            .finish()
    }
}

impl PartialEq for FramebufferTag {
    fn eq(&self, other: &Self) -> bool {
        ({ self.typ } == { other.typ }
            && { self.size } == { other.size }
            && { self.address } == { other.address }
            && { self.pitch } == { other.pitch }
            && { self.width } == { other.width }
            && { self.height } == { other.height }
            && { self.bpp } == { other.bpp }
            && { self.type_no } == { other.type_no }
            && self.buffer == other.buffer)
    }
}

/// Helper struct for [`FramebufferType`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
#[allow(clippy::upper_case_acronyms)]
enum FramebufferTypeId {
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

#[cfg(feature = "builder")]
impl<'a> FramebufferType<'a> {
    fn to_bytes(&self) -> Vec<u8> {
        let mut v = Vec::new();
        match self {
            FramebufferType::Indexed { palette } => {
                v.extend(0u8.to_le_bytes()); // type
                v.extend(0u16.to_le_bytes()); // reserved
                v.extend((palette.len() as u32).to_le_bytes());
                for color in palette.iter() {
                    v.extend(color.struct_as_bytes());
                }
            }
            FramebufferType::RGB { red, green, blue } => {
                v.extend(1u8.to_le_bytes()); // type
                v.extend(0u16.to_le_bytes()); // reserved
                v.extend(red.struct_as_bytes());
                v.extend(green.struct_as_bytes());
                v.extend(blue.struct_as_bytes());
            }
            FramebufferType::Text => {
                v.extend(2u8.to_le_bytes()); // type
                v.extend(0u16.to_le_bytes()); // reserved
            }
        }
        v
    }
}

/// An RGB color type field.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FramebufferField {
    /// Color field position.
    pub position: u8,

    /// Color mask size.
    pub size: u8,
}

#[cfg(feature = "builder")]
impl StructAsBytes for FramebufferField {
    fn byte_size(&self) -> usize {
        size_of::<Self>()
    }
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

#[cfg(feature = "builder")]
impl StructAsBytes for FramebufferColor {
    fn byte_size(&self) -> usize {
        size_of::<Self>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Compile time test
    #[test]
    fn test_size() {
        assert_eq!(size_of::<FramebufferColor>(), 3)
    }
}
