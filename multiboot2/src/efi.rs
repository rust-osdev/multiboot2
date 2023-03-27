//! All MBI tags related to (U)EFI.

use crate::TagType;
use crate::TagTypeId;
use core::convert::TryInto;
use core::mem::size_of;

#[cfg(feature = "builder")]
use crate::builder::traits::StructAsBytes;

/// EFI system table in 32 bit mode
#[derive(Clone, Copy, Debug)]
#[repr(C, packed)] // only repr(C) would add unwanted padding before first_section
pub struct EFISdt32 {
    typ: TagTypeId,
    size: u32,
    pointer: u32,
}

impl EFISdt32 {
    /// Create a new tag to pass the EFI32 System Table pointer.
    pub fn new(pointer: u32) -> Self {
        Self {
            typ: TagType::Efi32.into(),
            size: size_of::<Self>().try_into().unwrap(),
            pointer,
        }
    }

    /// The physical address of a i386 EFI system table.
    pub fn sdt_address(&self) -> usize {
        self.pointer as usize
    }
}

#[cfg(feature = "builder")]
impl StructAsBytes for EFISdt32 {
    fn byte_size(&self) -> usize {
        size_of::<Self>()
    }
}

/// EFI system table in 64 bit mode
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct EFISdt64 {
    typ: TagTypeId,
    size: u32,
    pointer: u64,
}

impl EFISdt64 {
    /// Create a new tag to pass the EFI64 System Table pointer.
    pub fn new(pointer: u64) -> Self {
        Self {
            typ: TagType::Efi64.into(),
            size: size_of::<Self>().try_into().unwrap(),
            pointer,
        }
    }

    /// The physical address of a x86_64 EFI system table.
    pub fn sdt_address(&self) -> usize {
        self.pointer as usize
    }
}

#[cfg(feature = "builder")]
impl StructAsBytes for EFISdt64 {
    fn byte_size(&self) -> usize {
        size_of::<Self>()
    }
}

/// Contains pointer to boot loader image handle.
#[derive(Debug)]
#[repr(C)]
pub struct EFIImageHandle32 {
    typ: TagTypeId,
    size: u32,
    pointer: u32,
}

impl EFIImageHandle32 {
    /// Returns the physical address of the EFI image handle.
    pub fn image_handle(&self) -> usize {
        self.pointer as usize
    }
}

/// Contains pointer to boot loader image handle.
#[derive(Debug)]
#[repr(C)]
pub struct EFIImageHandle64 {
    typ: TagTypeId,
    size: u32,
    pointer: u64,
}

impl EFIImageHandle64 {
    /// Returns the physical address of the EFI image handle.
    pub fn image_handle(&self) -> usize {
        self.pointer as usize
    }
}

#[cfg(test)]
mod tests {
    use super::{EFISdt32, EFISdt64};

    const ADDR: usize = 0xABCDEF;

    #[test]
    fn test_build_eftsdt32() {
        let tag = EFISdt32::new(ADDR.try_into().unwrap());
        assert_eq!(tag.sdt_address(), ADDR);
    }

    #[test]
    fn test_build_eftsdt64() {
        let tag = EFISdt64::new(ADDR.try_into().unwrap());
        assert_eq!(tag.sdt_address(), ADDR);
    }
}
