use core::fmt;

// Use this for implementing Debug on fields which are too large, or should not be printed.
// Although I haven't found anything explicitly saying that this will _definitely_ have the same
// representation as T, in my tests it always has. I don't know quite how alignment works either
// for this type, the rust docs aren't totally clear, and #[repr(C, packed)] is not valid.
#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
struct NoDebug<T>(T);

impl<T> fmt::Debug for NoDebug<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "...") // Write something, otherwise this looks funky in structs.
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct VBEInfoTag {
    typ: NoDebug<u32>,
    length: NoDebug<u32>,
    pub mode: u16,
    pub interface_segment: u16,
    pub interface_offset: u16,
    pub interface_length: u16,
    pub control_info: VBEControlInfo,
    pub mode_info: VBEModeInfo
}

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct VBEControlInfo {
    pub signature: [u8; 4],
    pub version: u16,
    pub oem_string_ptr: u32,
    pub capabilities: VBECapabilities,
    pub mode_list_ptr: u32,
    pub total_memory: u16, // Measured in 64kb blocks.
    pub oem_software_revision: u16,  
    pub oem_vendor_name_ptr: u32,
    pub oem_product_name_ptr: u32,
    pub oem_product_revision_ptr: u32,
    reserved: NoDebug<[u8; 222]>,
    oem_data: NoDebug<[u8; 256]>
}

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct VBEModeInfo {
    pub mode_attributes: VBEModeAttributes,
    pub window_a_attributes: VBEWindowAttributes,
    pub window_b_attributes: VBEWindowAttributes,
    pub window_granularity: u16, // Measured in kilobytes.
    pub window_size: u16,
    pub window_a_segment: u16,
    pub window_b_segment: u16,
    pub window_function_ptr: u32,
    pub pitch: u16,
    pub resolution: (u16, u16),
    pub character_size: (u8, u8),
    pub number_of_planes: u8,
    pub bpp: u8,
    pub number_of_banks: u8,
    pub memory_model: VBEMemoryModel,
    pub bank_size: u8, // Measured in kilobytes.
    pub number_of_image_pages: u8,
    reserved0: NoDebug<u8>,
    pub red_field: VBEField,
    pub green_field: VBEField,
    pub blue_field: VBEField,
    pub reserved_field: VBEField,
    pub direct_color_attributes: VBEDirectColorAttributes,
    pub framebuffer_base_ptr: u32,
    pub offscreen_memory_offset: u32,
    pub offscreen_memory_size: u16,
    reserved1: NoDebug<[u8; 206]>
}

#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(C, packed)]
pub struct VBEField {
    pub size: u8,
    pub position: u8
}

bitflags! {
    pub struct VBECapabilities: u32 {
        const SWITCHABLE_DAC = 0x1;     // The DAC can be switched between 6 and 8-bit modes.
        const NOT_VGA_COMPATIBLE = 0x2; // The controller can be switched into VGA modes.
        const RAMDAC_FIX = 0x4;         // When writing lots of information to the palette, the blank bit should be used.
    }
}

bitflags! {
    pub struct VBEModeAttributes: u16 {
        const SUPPORTED = 0x1;           // This mode is supported by the hardware.
        const TTY_SUPPORTED = 0x4;       // TTY output is supported.
        const COLOR = 0x8;
        const GRAPHICS = 0x10;
        const NOT_VGA_COMPATIBLE = 0x20;
        const NO_VGA_WINDOW = 0x40;      // If this is set, the window A and B fields of VBEModeInfo are invalid.
        const LINEAR_FRAMEBUFFER = 0x80; // A linear framebuffer is available for this mode.
    }
}

bitflags! {
    pub struct VBEWindowAttributes: u8 {
        const RELOCATABLE = 0x1;
        const READABLE = 0x2;
        const WRITEABLE = 0x4;
    }
}

bitflags! {
    pub struct VBEDirectColorAttributes: u8 {
        const PROGRAMMABLE = 0x1;       // The color ramp of the DAC is programmable.
        const RESERVED_USABLE = 0x2;    // The bits of the 'reserved' field are usable by the application.
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum VBEMemoryModel {
    Text = 0x00,
    CGAGraphics = 0x01,
    HerculesGraphics = 0x02,
    Planar = 0x03,
    PackedPixel = 0x04,
    Unchained = 0x05,
    DirectColor = 0x06,
    YUV = 0x07
}