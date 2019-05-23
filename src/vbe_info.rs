use core::fmt;

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct VBEInfoTag {
    typ: u32,
    length: u32,
    pub mode: u16,
    pub interface_segment: u16,
    pub interface_offset: u16,
    pub interface_length: u16,
    pub control_info: VBEControlInfo,
    pub mode_info: VBEModeInfo
}

#[derive(Copy, Clone)]
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
    reserved: [u8; 222],
    oem_data: [u8; 256]
}

impl fmt::Debug for VBEControlInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            f.debug_struct("VBEControlInfo")
             .field("signature", &self.signature)
             .field("version", &self.version)
             .field("oem_string_ptr", &self.oem_string_ptr)
             .field("capabilities", &self.capabilities)
             .field("mode_list_ptr", &self.mode_list_ptr)
             .field("total_memory", &self.total_memory)
             .field("oem_software_revision", &self.oem_software_revision)
             .field("oem_vendor_name_ptr", &self.oem_vendor_name_ptr)
             .field("oem_product_name_ptr", &self.oem_product_name_ptr)
             .field("oem_product_revision_ptr", &self.oem_product_revision_ptr)
             .finish()
        }
    }
}

#[derive(Copy, Clone)]
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
    reserved0: u8,
    pub red_field: VBEField,
    pub green_field: VBEField,
    pub blue_field: VBEField,
    pub reserved_field: VBEField,
    pub direct_color_attributes: VBEDirectColorAttributes,
    pub framebuffer_base_ptr: u32,
    pub offscreen_memory_offset: u32,
    pub offscreen_memory_size: u16,
    reserved1: [u8; 206]
}

impl fmt::Debug for VBEModeInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            f.debug_struct("VBEModeInfo")
             .field("mode_attributes", &self.mode_attributes)
             .field("window_a_attributes", &self.window_a_attributes)
             .field("window_b_attributes", &self.window_b_attributes)
             .field("window_granularity", &self.window_granularity)
             .field("window_size", &self.window_size)
             .field("window_a_segment", &self.window_a_segment)
             .field("window_b_segment", &self.window_b_segment)
             .field("window_function_ptr", &self.window_function_ptr)
             .field("pitch", &self.pitch)
             .field("resolution", &self.resolution)
             .field("character_size", &self.character_size)
             .field("number_of_planes", &self.number_of_planes)
             .field("bpp", &self.bpp)
             .field("number_of_banks", &self.number_of_banks)
             .field("memory_model", &self.memory_model)
             .field("bank_size", &self.bank_size)
             .field("number_of_image_pages", &self.number_of_image_pages)
             .field("red_field", &self.red_field)
             .field("green_field", &self.green_field)
             .field("blue_field", &self.blue_field)
             .field("reserved_field", &self.reserved_field)
             .field("direct_color_attributes", &self.direct_color_attributes)
             .field("framebuffer_base_ptr", &self.framebuffer_base_ptr)
             .field("offscreen_memory_offset", &self.offscreen_memory_offset)
             .field("offscreen_memory_size", &self.offscreen_memory_size)
             .finish()
        }
    }
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