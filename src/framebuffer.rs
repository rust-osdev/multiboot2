use core::ptr;
use core::slice;
use header::Tag;

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

#[derive(Debug, PartialEq)]
#[repr(C, packed)]
pub struct FramebufferColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8
}

struct Reader {
    ptr: *const u8,
    off: usize
}

impl Reader {
    fn new<T>(ptr: *const T) -> Reader {
        Reader {
            ptr: ptr as *const u8,
            off: 0
        }
    }

    fn read_u8(&mut self) -> u8 {
        self.off += 1;
        unsafe {
            ptr::read(self.ptr.offset((self.off - 1) as isize))
        }
    }

    fn read_u16(&mut self) -> u16 {
        self.read_u8() as u16 | (self.read_u8() as u16) << 8
    }

    fn read_u32(&mut self) -> u32 {
        self.read_u16() as u32 | (self.read_u16() as u32) << 16
    }

    fn read_u64(&mut self) -> u64 {
        self.read_u32() as u64 | (self.read_u32() as u64) << 32
    }

    fn skip(&mut self, n: usize) {
        self.off += n;
    }
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
                slice::from_raw_parts(reader.ptr.offset(reader.off as isize) as *const FramebufferColor, num_colors as usize)
            } as &'static [FramebufferColor];
            FramebufferType::Indexed { palette }
        },
        1 => {
            let red_pos = reader.read_u8();     //These refer to the bit positions of the MSB of each field
            let red_mask = reader.read_u8();    //And then the length of the field from MSB to LSB
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
