/// The tag that the bootloader passes will depend on the ACPI version the hardware supports.
/// For ACPI Version 1.0, a `RsdpV1Tag` will be provided, which can be accessed from
/// `BootInformation` using the `rsdp_v1_tag` function. For subsequent versions of ACPI, a
/// `RsdpV2Tag` will be provided, which can be accessed with `rsdp_v2_tag`.
///
/// Even though the bootloader should give the address of the real RSDP/XSDT, the checksum and
/// signature should be manually verified.

use core::slice;
use core::str;

const RSDPV1_LENGTH: usize = 20;

/// EFI system table in 32 bit mode
#[derive(Clone, Copy, Debug)]
#[repr(C, packed)] // only repr(C) would add unwanted padding before first_section
pub struct EFISdt32 {
    typ: u32,
    size: u32,
    pointer: u32,
}

impl EFISdt32 {
    /// The Physical address of a i386 EFI system table.
    pub fn sdt_address(&self) -> usize {
        self.pointer as usize
    }
}

/// EFI system table in 64 bit mode
#[derive(Clone, Copy, Debug)]
#[repr(C)] 
pub struct EFISdt64 {
    typ: u32,
    size: u32,
    pointer: u64,
}

impl EFISdt64 {
    /// The Physical address of a x86_64 EFI system table.
    pub fn sdt_address(&self) -> usize {
        self.pointer as usize
    }
}

/// Contains pointer to boot loader image handle.
#[derive(Debug)]
#[repr(C)]
pub struct EFIImageHandle32 {
    typ: u32,
    size: u32,
    pointer: u32,
}

/// Contains pointer to boot loader image handle.
#[derive(Debug)]
#[repr(C)]
pub struct EFIImageHandle64 {
    typ: u32,
    size: u32,
    pointer: u64,
}

/// If the image has relocatable header tag, this tag contains the image's 
/// base physical address.
#[derive(Debug)]
#[repr(C)]
pub struct ImageLoadPhysAddr {
    typ: u32,
    size: u32,
    load_base_addr: u32,
}

/// This tag contains a copy of RSDP as defined per ACPI 1.0 specification. 
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
    /// The "RSD PTR " marker singature.
    ///
    /// This is originally a 8-byte C string (not null terminated!) that must contain "RSD PTR "
    pub fn signature<'a>(&'a self) -> Option<&'a str> {
        str::from_utf8(&self.signature).ok()
    }

    /// Validation of the RSDPv1 checksum
    pub fn checksum_is_valid(&self) -> bool {
        let bytes = unsafe {
            slice::from_raw_parts(
                self as *const _ as *const u8,
                RSDPV1_LENGTH + 8
            )
        };
        bytes[8..].iter().fold(0u8, |acc, val| acc.wrapping_add(*val)) == 0
    }

    /// An OEM-supplied string that identifies the OEM. 
    pub fn oem_id<'a>(&'a self) -> Option<&'a str> {
        str::from_utf8(&self.oem_id).ok()
    }

    /// The revision of the ACPI.
    pub fn revision(&self) -> u8 {
        self.revision
    }

    /// The physical (I repeat: physical) address of the RSDT table.
    pub fn rsdt_address(&self) -> usize {
        self.rsdt_address as usize
    }
}

/// This tag contains a copy of RSDP as defined per ACPI 2.0 or later specification. 
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
    /// The "RSD PTR " marker singature.
    ///
    /// This is originally a 8-byte C string (not null terminated!) that must contain "RSD PTR ".
    pub fn signature<'a>(&'a self) -> Option<&'a str> {
        str::from_utf8(&self.signature).ok()
    }

    /// Validation of the RSDPv2 extended checksum
    pub fn checksum_is_valid(&self) -> bool {
        let bytes = unsafe {
            slice::from_raw_parts(
                self as *const _ as *const u8,
                self.length as usize + 8
            )
        };
        bytes[8..].iter().fold(0u8, |acc, val| acc.wrapping_add(*val)) == 0
    }

    /// An OEM-supplied string that identifies the OEM. 
    pub fn oem_id<'a>(&'a self) -> Option<&'a str> {
        str::from_utf8(&self.oem_id).ok()
    }

    /// The revision of the ACPI.
    pub fn revision(&self) -> u8 {
        self.revision
    }

    /// Physical address of the XSDT table.
    ///
    /// On x86, this is truncated from 64-bit to 32-bit.
    pub fn xsdt_address(&self) -> usize {
        self.xsdt_address as usize
    }

    /// This field is used to calculate the checksum of the entire table, including both checksum fields. 
    pub fn ext_checksum(&self) -> u8 {
        self.ext_checksum
    }
}
