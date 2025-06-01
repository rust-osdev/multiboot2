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

/// Helper struct to read bytes from a raw pointer and increase the pointer
/// automatically.
struct Reader<'a> {
    buffer: &'a [u8],
    off: usize,
}

impl<'a> Reader<'a> {
    const fn new(buffer: &'a [u8]) -> Self {
        Self { buffer, off: 0 }
    }

    /// Reads the next [`u8`] from the buffer and updates the internal pointer.
    ///
    /// # Panic
    ///
    /// Panics if the index is out of bounds.
    fn read_next_u8(&mut self) -> u8 {
        let val = self
            .buffer
            .get(self.off)
            .cloned()
            // This is not a solution I'm proud of, but at least it is safe.
            // The whole framebuffer tag code originally is not from me.
            // I hope someone from the community wants to improve this overall
            // functionality someday.
            .expect("Embedded framebuffer info should be properly sized and available");
        self.off += 1;
        val
    }

    /// Reads the next [`u16`] from the buffer and updates the internal pointer.
    ///
    /// # Panic
    ///
    /// Panics if the index is out of bounds.
    fn read_next_u16(&mut self) -> u16 {
        let u16_lo = self.read_next_u8() as u16;
        let u16_hi = self.read_next_u8() as u16;
        (u16_hi << 8) | u16_lo
    }

    const fn current_ptr(&self) -> *const u8 {
        unsafe { self.buffer.as_ptr().add(self.off) }
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

    /// The type of framebuffer. See [`FramebufferTypeId`].
    // TODO: Strictly speaking this causes UB for invalid values. However, no
    //  sane bootloader puts something illegal there at the moment. When we
    //  refactor this (newtype pattern?), we should also streamline other
    //  parts in the code base accordingly.
    framebuffer_type: FramebufferTypeId,

    _padding: u16,

    /// This optional data and its meaning depend on the [`FramebufferTypeId`].
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
        buffer_type: FramebufferType,
    ) -> Box<Self> {
        let header = TagHeader::new(Self::ID, 0);
        let address = address.to_ne_bytes();
        let pitch = pitch.to_ne_bytes();
        let width = width.to_ne_bytes();
        let height = height.to_ne_bytes();
        let buffer_type_id = buffer_type.id();
        let padding = [0; 2];
        let optional_buffer = buffer_type.serialize();
        new_boxed(
            header,
            &[
                &address,
                &pitch,
                &width,
                &height,
                &[bpp],
                &[buffer_type_id as u8],
                &padding,
                &optional_buffer,
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

    /// The type of framebuffer, one of: `Indexed`, `RGB` or `Text`.
    pub fn buffer_type(&self) -> Result<FramebufferType, UnknownFramebufferType> {
        let mut reader = Reader::new(&self.buffer);

        // TODO: We should use the newtype pattern instead or so to properly
        //  solve this.
        let fb_type_raw = self.framebuffer_type as u8;
        let fb_type = FramebufferTypeId::try_from(fb_type_raw)?;

        match fb_type {
            FramebufferTypeId::Indexed => {
                // TODO we can create a struct for this and implement
                //  DynSizedStruct for it to leverage the already existing
                //  functionality
                let num_colors = reader.read_next_u16();

                let palette = {
                    // Ensure the slice can be created without causing UB
                    assert_eq!(mem::size_of::<FramebufferColor>(), 3);

                    unsafe {
                        slice::from_raw_parts(
                            reader.current_ptr().cast::<FramebufferColor>(),
                            num_colors as usize,
                        )
                    }
                };
                Ok(FramebufferType::Indexed { palette })
            }
            FramebufferTypeId::RGB => {
                let red_pos = reader.read_next_u8(); // These refer to the bit positions of the LSB of each field
                let red_mask = reader.read_next_u8(); // And then the length of the field from LSB to MSB
                let green_pos = reader.read_next_u8();
                let green_mask = reader.read_next_u8();
                let blue_pos = reader.read_next_u8();
                let blue_mask = reader.read_next_u8();
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
            .field("buffer_type", &self.buffer_type())
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
            && self.framebuffer_type == { other.framebuffer_type }
            && self.buffer == other.buffer
    }
}

/// ABI-compatible framebuffer type.
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

/// Structured accessory to the provided framebuffer type that is not ABI
/// compatible.
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

impl FramebufferType<'_> {
    #[must_use]
    #[cfg(feature = "builder")]
    const fn id(&self) -> FramebufferTypeId {
        match self {
            FramebufferType::Indexed { .. } => FramebufferTypeId::Indexed,
            FramebufferType::RGB { .. } => FramebufferTypeId::RGB,
            FramebufferType::Text => FramebufferTypeId::Text,
        }
    }

    #[must_use]
    #[cfg(feature = "builder")]
    fn serialize(&self) -> alloc::vec::Vec<u8> {
        let mut data = alloc::vec::Vec::new();
        match self {
            FramebufferType::Indexed { palette } => {
                // TODO we can create a struct for this and implement
                //  DynSizedStruct for it to leverage the already existing
                //  functionality
                let num_colors = palette.len() as u16;
                data.extend(&num_colors.to_ne_bytes());
                for color in *palette {
                    let serialized_color = [color.red, color.green, color.blue];
                    data.extend(&serialized_color);
                }
            }
            FramebufferType::RGB { red, green, blue } => data.extend(&[
                red.position,
                red.size,
                green.position,
                green.size,
                blue.position,
                blue.size,
            ]),
            FramebufferType::Text => {}
        }
        data
    }
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
///
/// On the ABI level, multiple values are consecutively without padding bytes.
/// The spec is not precise in that regard, but looking at Limine's and GRUB's
/// source code confirm that.
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
#[display("Unknown framebuffer type {}", _0)]
pub struct UnknownFramebufferType(u8);

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
        let tag = FramebufferTag::new(0x1000, 1, 1024, 1024, 8, FramebufferType::Text);
        // Good test for Miri
        dbg!(tag);

        let tag = FramebufferTag::new(
            0x1000,
            1,
            1024,
            1024,
            8,
            FramebufferType::Indexed {
                palette: &[
                    FramebufferColor {
                        red: 255,
                        green: 255,
                        blue: 255,
                    },
                    FramebufferColor {
                        red: 127,
                        green: 42,
                        blue: 73,
                    },
                ],
            },
        );
        // Good test for Miri
        dbg!(tag);

        let tag = FramebufferTag::new(
            0x1000,
            1,
            1024,
            1024,
            8,
            FramebufferType::RGB {
                red: FramebufferField {
                    position: 0,
                    size: 0,
                },
                green: FramebufferField {
                    position: 10,
                    size: 20,
                },
                blue: FramebufferField {
                    position: 30,
                    size: 40,
                },
            },
        );
        // Good test for Miri
        dbg!(tag);
    }
}
