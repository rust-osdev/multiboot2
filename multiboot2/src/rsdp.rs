//! Module for [`RsdpV1Tag`] and  [`RsdpV2Tag`].

//! Module for RSDP/ACPI. RSDP (Root System Description Pointer) is a data structure used in the
//! ACPI programming interface.
//!
//! The tag that the bootloader passes will depend on the ACPI version the hardware supports.
//! For ACPI Version 1.0, a `RsdpV1Tag` will be provided, which can be accessed from
//! `BootInformation` using the `rsdp_v1_tag` function. For subsequent versions of ACPI, a
//! `RsdpV2Tag` will be provided, which can be accessed with `rsdp_v2_tag`.
//!
//! Even though the bootloader should give the address of the real RSDP/XSDT, the checksum and
//! signature should be manually verified.
//!

use crate::TagType;
use crate::tag::TagHeader;
use core::slice;
use core::str;
use core::str::Utf8Error;
use multiboot2_common::{MaybeDynSized, Tag};

fn sum_bytes(parts: &[&[u8]]) -> u8 {
    parts
        .iter()
        .flat_map(|part| part.iter().copied())
        .fold(0u8, |acc, val| acc.wrapping_add(val))
}

fn compute_checksum(bytes: &[&[u8]]) -> u8 {
    0u8.wrapping_sub(sum_bytes(bytes))
}

/// This tag contains a copy of RSDP as defined per ACPI 1.0 specification.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C, align(8))]
pub struct RsdpV1Tag {
    header: TagHeader,
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    // This is the PHYSICAL address of the RSDT
    rsdt_address: u32,
}

impl RsdpV1Tag {
    /// Signature of RSDP v1.
    pub const SIGNATURE: [u8; 8] = *b"RSD PTR ";

    const BASE_SIZE: usize = size_of::<TagHeader>() + 16 + 4;

    /// Constructs a new tag.
    #[must_use]
    pub fn new(oem_id: [u8; 6], revision: u8, rsdt_address: u32) -> Self {
        let rsdt_address_bytes = rsdt_address.to_le_bytes();
        let checksum_seed = [0u8];
        let revision_bytes = [revision];
        let checksum = compute_checksum(&[
            Self::SIGNATURE.as_slice(),
            checksum_seed.as_slice(),
            oem_id.as_slice(),
            revision_bytes.as_slice(),
            rsdt_address_bytes.as_slice(),
        ]);
        Self {
            header: TagHeader::new(Self::ID, Self::BASE_SIZE as u32),
            signature: Self::SIGNATURE,
            checksum,
            oem_id,
            revision,
            rsdt_address,
        }
    }

    /// The "RSD PTR " marker signature.
    ///
    /// This is originally a 8-byte C string (not null terminated!) that must contain "RSD PTR "
    pub const fn signature(&self) -> Result<&str, Utf8Error> {
        str::from_utf8(&self.signature)
    }

    /// Validation of the RSDPv1 checksum.
    #[must_use]
    pub fn checksum_is_valid(&self) -> bool {
        let rsdp_ptr = (self as *const Self)
            .cast::<u8>()
            // Skip header
            .wrapping_add(size_of::<TagHeader>());
        let rsdp_len = Self::BASE_SIZE - size_of::<TagHeader>();
        // SAFETY: `self` is a valid reference, and we only read the
        // initialized raw representation of the fixed-size RSDP payload.
        let bytes = unsafe { slice::from_raw_parts(rsdp_ptr, rsdp_len) };
        sum_bytes(&[bytes]) == 0
    }

    /// An OEM-supplied string that identifies the OEM.
    pub const fn oem_id(&self) -> Result<&str, Utf8Error> {
        str::from_utf8(&self.oem_id)
    }

    /// The revision of the ACPI.
    #[must_use]
    pub const fn revision(&self) -> u8 {
        self.revision
    }

    /// The physical (I repeat: physical) address of the RSDT table.
    #[must_use]
    pub const fn rsdt_address(&self) -> usize {
        self.rsdt_address as usize
    }
}

impl MaybeDynSized for RsdpV1Tag {
    type Header = TagHeader;

    const BASE_SIZE: usize = size_of::<Self>();
}

impl Tag for RsdpV1Tag {
    type IDType = TagType;

    const ID: TagType = TagType::AcpiV1;
}

/// This tag contains a copy of RSDP as defined per ACPI 2.0 or later specification.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C, align(8))]
pub struct RsdpV2Tag {
    header: TagHeader,
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_address: u32,
    length: u32,
    // This is the PHYSICAL address of the XSDT
    xsdt_address: u64,
    ext_checksum: u8,
    _reserved: [u8; 3],
}

impl RsdpV2Tag {
    /// Signature of RSDP v2.
    pub const SIGNATURE: [u8; 8] = *b"RSD PTR ";

    const BASE_SIZE: usize =
        size_of::<TagHeader>() + 16 + 2 * size_of::<u32>() + size_of::<u64>() + 4;

    /// Constructs a new tag.
    #[must_use]
    pub fn new(
        oem_id: [u8; 6],
        revision: u8,
        rsdt_address: u32,
        length: u32,
        xsdt_address: u64,
    ) -> Self {
        let rsdt_address_bytes = rsdt_address.to_le_bytes();
        let length_bytes = length.to_le_bytes();
        let xsdt_address_bytes = xsdt_address.to_le_bytes();
        let checksum_seed = [0u8];
        let ext_checksum_seed = [0u8];
        let reserved = [0u8; 3];
        let revision_bytes = [revision];
        let checksum = compute_checksum(&[
            Self::SIGNATURE.as_slice(),
            checksum_seed.as_slice(),
            oem_id.as_slice(),
            revision_bytes.as_slice(),
            rsdt_address_bytes.as_slice(),
        ]);
        let ext_checksum = compute_checksum(&[
            Self::SIGNATURE.as_slice(),
            core::slice::from_ref(&checksum),
            oem_id.as_slice(),
            revision_bytes.as_slice(),
            rsdt_address_bytes.as_slice(),
            length_bytes.as_slice(),
            xsdt_address_bytes.as_slice(),
            ext_checksum_seed.as_slice(),
            reserved.as_slice(),
        ]);
        Self {
            header: TagHeader::new(Self::ID, Self::BASE_SIZE as u32),
            signature: Self::SIGNATURE,
            checksum,
            oem_id,
            revision,
            rsdt_address,
            length,
            xsdt_address,
            ext_checksum,
            _reserved: [0; 3],
        }
    }

    /// The "RSD PTR " marker signature.
    ///
    /// This is originally a 8-byte C string (not null terminated!) that must contain "RSD PTR ".
    pub const fn signature(&self) -> Result<&str, Utf8Error> {
        str::from_utf8(&self.signature)
    }

    /// Validation of the RSDPv2 extended checksum.
    #[must_use]
    pub fn checksum_is_valid(&self) -> bool {
        // SAFETY: `self` is a valid reference, and we only read the
        // initialized raw representation of the fixed-size layout.
        let bytes =
            unsafe { slice::from_raw_parts((self as *const Self).cast::<u8>(), size_of::<Self>()) };
        let length = self.length as usize;
        if length != Self::BASE_SIZE - size_of::<TagHeader>() {
            return false;
        }
        let ext_end = size_of::<TagHeader>() + length;
        if ext_end > bytes.len() {
            return false;
        }

        sum_bytes(&[&bytes[8..28]]) == 0 && sum_bytes(&[&bytes[8..ext_end]]) == 0
    }

    /// An OEM-supplied string that identifies the OEM.
    pub const fn oem_id(&self) -> Result<&str, Utf8Error> {
        str::from_utf8(&self.oem_id)
    }

    /// The revision of the ACPI.
    #[must_use]
    pub const fn revision(&self) -> u8 {
        self.revision
    }

    /// Physical address of the XSDT table.
    ///
    /// On x86, this is truncated from 64-bit to 32-bit.
    #[must_use]
    pub const fn xsdt_address(&self) -> usize {
        self.xsdt_address as usize
    }

    /// This field is used to calculate the checksum of the entire table, including both checksum fields.
    #[must_use]
    pub const fn ext_checksum(&self) -> u8 {
        self.ext_checksum
    }
}

impl MaybeDynSized for RsdpV2Tag {
    type Header = TagHeader;

    const BASE_SIZE: usize = size_of::<Self>();
}

impl Tag for RsdpV2Tag {
    type IDType = TagType;

    const ID: TagType = TagType::AcpiV2;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v1_new_computes_valid_checksum() {
        let tag = RsdpV1Tag::new(*b"ABCDEF", 1, 0x1234_5678);
        assert!(tag.checksum_is_valid());
    }

    #[test]
    fn v2_new_computes_valid_checksums() {
        let tag = RsdpV2Tag::new(*b"ABCDEF", 2, 0x1234_5678, 36, 0x1234_5678_9abc_def0);
        assert!(tag.checksum_is_valid());
    }

    #[test]
    fn v2_checksum_validation_rejects_corruption() {
        let mut tag = RsdpV2Tag::new(*b"ABCDEF", 2, 0x1234_5678, 36, 0x1234_5678_9abc_def0);
        tag.ext_checksum ^= 1;
        assert!(!tag.checksum_is_valid());
    }

    #[test]
    fn v2_checksum_validation_rejects_checksum_corruption() {
        let mut tag = RsdpV2Tag::new(*b"ABCDEF", 2, 0x1234_5678, 36, 0x1234_5678_9abc_def0);
        tag.checksum ^= 1;
        assert!(!tag.checksum_is_valid());
    }

    #[test]
    fn v2_checksum_validation_rejects_invalid_length() {
        let mut tag = RsdpV2Tag::new(*b"ABCDEF", 2, 0x1234_5678, 36, 0x1234_5678_9abc_def0);
        tag.length = 0;
        assert!(!tag.checksum_is_valid());
    }
}
