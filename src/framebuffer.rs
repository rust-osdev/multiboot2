use header::Tag;
use ::Reader;
use core::slice;

#[derive(Debug, PartialEq)]
pub struct FramebufferTag<'a> {
    pub address: u64,
    pub pitch: u32,
    pub width: u32,
    pub height: u32,
    pub bpp: u8,
    pub buffer_type: FramebufferType<'a>
}

#[derive(Debug, PartialEq)]
pub enum FramebufferType<'a> {
    Indexed {
        palette: &'a [FramebufferColor]
    },
    RGB {
        red: FramebufferField,
        green: FramebufferField,
        blue: FramebufferField
    },
    Text
}

#[derive(Debug, PartialEq)]
pub struct FramebufferField {
    pub position: u8,
    pub size: u8
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C, packed)]
pub struct FramebufferColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8
}

pub fn framebuffer_tag<'a>(tag: &'a Tag) -> FramebufferTag<'a> {
    let mut reader = Reader::new(tag as *const Tag);
    reader.skip(8);
    let address = reader.read_u64();
    let pitch = reader.read_u32();
    let width = reader.read_u32();
    let height = reader.read_u32();
    let bpp = reader.read_u8();
    let type_no = reader.read_u8();
    reader.skip(2); // In the multiboot spec, it has this listed as a u8 _NOT_ a u16.
                    // Reading the GRUB2 source code reveals it is in fact a u16.
    let buffer_type = match type_no {
        0 =>  {
            let num_colors = reader.read_u32();
            let palette = unsafe {
                slice::from_raw_parts(reader.current_address() as *const FramebufferColor, num_colors as usize)
            } as &'static [FramebufferColor];
            FramebufferType::Indexed { palette }
        },
        1 => {
            let red_pos = reader.read_u8();     // These refer to the bit positions of the LSB of each field
            let red_mask = reader.read_u8();    // And then the length of the field from LSB to MSB
            let green_pos = reader.read_u8();   
            let green_mask = reader.read_u8();  
            let blue_pos = reader.read_u8();
            let blue_mask = reader.read_u8();
            FramebufferType::RGB {
                red: FramebufferField { position: red_pos, size: red_mask },
                green: FramebufferField { position: green_pos, size: green_mask },
                blue: FramebufferField { position: blue_pos, size: blue_mask }
            }
        },
        2 => FramebufferType::Text,
        _ => panic!("Unknown framebuffer type: {}", type_no)
    };

    FramebufferTag {
        address,
        pitch,
        width,
        height,
        bpp,
        buffer_type
    }
}
