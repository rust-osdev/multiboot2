/// The tag that the bootloader passes will depend on the ACPI version the hardware supports.
/// For ACPI Version 1.0, a `RsdpV1Tag` will be provided, which can be accessed from
/// `BootInformation` using the `rsdp_v1_tag` function. For subsequent versions of ACPI, a
/// `RsdpV2Tag` will be provided, which can be accessed with `rsdp_v2_tag`.
///
/// Even though the bootloader should give the address of the real RSDP/XSDT, the checksum and
/// signature should be manually verified.

use core::str;

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct RsdpV1Tag {
    typ: u32,
    size: u32,
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_address: u32,  // This is the PHYSICAL address of the RSDT
}

impl RsdpV1Tag {
    pub fn signature<'a>(&'a self) -> Option<&'a str> {
        str::from_utf8(&self.signature).ok()
    }

    pub fn checksum(&self) -> u8 {
        self.checksum
    }

    pub fn oem_id<'a>(&'a self) -> Option<&'a str> {
        str::from_utf8(&self.oem_id).ok()
    }

    pub fn revision(&self) -> u8 {
        self.revision
    }

    /// Get the physical address of the RSDT.
    pub fn rsdt_address(&self) -> usize {
        self.rsdt_address as usize
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct RsdpV2Tag {
    typ: u32,
    size: u32,
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    _rsdt_address: u32,
    length: u32,
    xsdt_address: u64,  // This is the PHYSICAL address of the XSDT
    ext_checksum: u8,
    _reserved: [u8; 3],
}

impl RsdpV2Tag {
    pub fn signature<'a>(&'a self) -> Option<&'a str> {
        str::from_utf8(&self.signature).ok()
    }

    pub fn checksum(&self) -> u8 {
        self.checksum
    }

    pub fn oem_id<'a>(&'a self) -> Option<&'a str> {
        str::from_utf8(&self.oem_id).ok()
    }

    pub fn revision(&self) -> u8 {
        self.revision
    }

    /// Get the physical address of the XSDT. On x86, this is truncated from 64-bit to 32-bit.
    pub fn xsdt_address(&self) -> usize {
        self.xsdt_address as usize
    }

    pub fn ext_checksum(&self) -> u8 {
        self.ext_checksum
    }
}
