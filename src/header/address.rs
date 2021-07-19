use core::mem::size_of;
use {HeaderTagFlag, HeaderTagType};

/// This information does not need to be provided if the kernel image is in ELF
/// format, but it must be provided if the image is in a.out format or in some
/// other format. Required for legacy boot (BIOS).
/// Determines load addresses.
#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct AddressHeaderTag {
    typ: HeaderTagType,
    flags: HeaderTagFlag,
    size: u32,
    /// Contains the address corresponding to the beginning of the Multiboot2 header — the physical memory location at which the magic value is supposed to be loaded. This field serves to synchronize the mapping between OS image offsets and physical memory addresses.
    header_addr: u32,
    /// Contains the physical address of the beginning of the text segment. The offset in the OS image file at which to start loading is defined by the offset at which the header was found, minus (header_addr - load_addr). load_addr must be less than or equal to header_addr.
    ///
    /// Special value -1 means that the file must be loaded from its beginning.
    load_addr: u32,
    /// Contains the physical address of the end of the data segment. (load_end_addr - load_addr) specifies how much data to load. This implies that the text and data segments must be consecutive in the OS image; this is true for existing a.out executable formats. If this field is zero, the boot loader assumes that the text and data segments occupy the whole OS image file.
    load_end_addr: u32,
    /// Contains the physical address of the end of the bss segment. The boot loader initializes this area to zero, and reserves the memory it occupies to avoid placing boot modules and other data relevant to the operating system in that area. If this field is zero, the boot loader assumes that no bss segment is present.
    bss_end_addr: u32,
}

impl AddressHeaderTag {
    pub fn new(
        flags: HeaderTagFlag,
        header_addr: u32,
        load_addr: u32,
        load_end_addr: u32,
        bss_end_addr: u32,
    ) -> Self {
        AddressHeaderTag {
            typ: HeaderTagType::Address,
            flags,
            size: size_of::<Self>() as u32,
            header_addr,
            load_addr,
            load_end_addr,
            bss_end_addr,
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
    pub fn header_addr(&self) -> u32 {
        self.header_addr
    }
    pub fn load_addr(&self) -> u32 {
        self.load_addr
    }
    pub fn load_end_addr(&self) -> u32 {
        self.load_end_addr
    }
    pub fn bss_end_addr(&self) -> u32 {
        self.bss_end_addr
    }
}
