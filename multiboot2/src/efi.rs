//! All MBI tags related to (U)EFI.

use crate::TagTypeId;
use crate::{Tag, TagTrait, TagType};
use core::convert::TryInto;
use core::mem::size_of;

/// EFI system table in 32 bit mode tag.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct EFISdt32Tag {
    typ: TagTypeId,
    size: u32,
    pointer: u32,
}

impl EFISdt32Tag {
    /// Create a new tag to pass the EFI32 System Table pointer.
    pub fn new(pointer: u32) -> Self {
        Self {
            typ: Self::ID.into(),
            size: size_of::<Self>().try_into().unwrap(),
            pointer,
        }
    }

    /// The physical address of a i386 EFI system table.
    pub fn sdt_address(&self) -> usize {
        self.pointer as usize
    }
}

impl TagTrait for EFISdt32Tag {
    const ID: TagType = TagType::Efi32;

    fn dst_size(_base_tag: &Tag) {}
}

/// EFI system table in 64 bit mode tag.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct EFISdt64Tag {
    typ: TagTypeId,
    size: u32,
    pointer: u64,
}

impl EFISdt64Tag {
    /// Create a new tag to pass the EFI64 System Table pointer.
    pub fn new(pointer: u64) -> Self {
        Self {
            typ: Self::ID.into(),
            size: size_of::<Self>().try_into().unwrap(),
            pointer,
        }
    }

    /// The physical address of a x86_64 EFI system table.
    pub fn sdt_address(&self) -> usize {
        self.pointer as usize
    }
}

impl TagTrait for EFISdt64Tag {
    const ID: TagType = TagType::Efi64;

    fn dst_size(_base_tag: &Tag) {}
}

/// Tag that contains the pointer to the boot loader's UEFI image handle
/// (32-bit).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct EFIImageHandle32Tag {
    typ: TagTypeId,
    size: u32,
    pointer: u32,
}

impl EFIImageHandle32Tag {
    #[cfg(feature = "builder")]
    pub fn new(pointer: u32) -> Self {
        Self {
            typ: Self::ID.into(),
            size: size_of::<Self>().try_into().unwrap(),
            pointer,
        }
    }

    /// Returns the physical address of the EFI image handle.
    pub fn image_handle(&self) -> usize {
        self.pointer as usize
    }
}

impl TagTrait for EFIImageHandle32Tag {
    const ID: TagType = TagType::Efi32Ih;

    fn dst_size(_base_tag: &Tag) {}
}

/// Tag that contains the pointer to the boot loader's UEFI image handle
/// (64-bit).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct EFIImageHandle64Tag {
    typ: TagTypeId,
    size: u32,
    pointer: u64,
}

impl EFIImageHandle64Tag {
    #[cfg(feature = "builder")]
    pub fn new(pointer: u64) -> Self {
        Self {
            typ: Self::ID.into(),
            size: size_of::<Self>().try_into().unwrap(),
            pointer,
        }
    }

    /// Returns the physical address of the EFI image handle.
    pub fn image_handle(&self) -> usize {
        self.pointer as usize
    }
}

impl TagTrait for EFIImageHandle64Tag {
    const ID: TagType = TagType::Efi64Ih;

    fn dst_size(_base_tag: &Tag) {}
}

/// EFI ExitBootServices was not called tag.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct EFIBootServicesNotExitedTag {
    typ: TagTypeId,
    size: u32,
}

impl EFIBootServicesNotExitedTag {
    #[cfg(feature = "builder")]
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(feature = "builder")]
impl Default for EFIBootServicesNotExitedTag {
    fn default() -> Self {
        Self {
            typ: TagType::EfiBs.into(),
            size: core::mem::size_of::<Self>().try_into().unwrap(),
        }
    }
}

impl TagTrait for EFIBootServicesNotExitedTag {
    const ID: TagType = TagType::EfiBs;

    fn dst_size(_base_tag: &Tag) {}
}

#[cfg(all(test, feature = "builder"))]
mod tests {
    use super::{EFIImageHandle32Tag, EFIImageHandle64Tag, EFISdt32Tag, EFISdt64Tag};

    const ADDR: usize = 0xABCDEF;

    #[test]
    fn test_build_eftsdt32() {
        let tag = EFISdt32Tag::new(ADDR.try_into().unwrap());
        assert_eq!(tag.sdt_address(), ADDR);
    }

    #[test]
    fn test_build_eftsdt64() {
        let tag = EFISdt64Tag::new(ADDR.try_into().unwrap());
        assert_eq!(tag.sdt_address(), ADDR);
    }

    #[test]
    fn test_build_eftih32() {
        let tag = EFIImageHandle32Tag::new(ADDR.try_into().unwrap());
        assert_eq!(tag.image_handle(), ADDR);
    }

    #[test]
    fn test_build_eftih64() {
        let tag = EFIImageHandle64Tag::new(ADDR.try_into().unwrap());
        assert_eq!(tag.image_handle(), ADDR);
    }
}
