//! All tags related to (U)EFI with the exception of EFI memory tags:
//!
//! - [`EFISdt32Tag`]
//! - [`EFISdt64Tag`]
//! - [`EFIImageHandle32Tag`]
//! - [`EFIImageHandle64Tag`]
//! - [`EFIBootServicesNotExitedTag`]

use crate::tag::TagHeader;
use crate::{TagTrait, TagType};
use core::mem::size_of;

/// EFI system table in 32 bit mode tag.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C, align(8))]
pub struct EFISdt32Tag {
    header: TagHeader,
    pointer: u32,
}

impl EFISdt32Tag {
    /// Create a new tag to pass the EFI32 System Table pointer.
    #[must_use]
    pub fn new(pointer: u32) -> Self {
        Self {
            header: TagHeader::new(Self::ID, size_of::<Self>().try_into().unwrap()),
            pointer,
        }
    }

    /// The physical address of a i386 EFI system table.
    #[must_use]
    pub const fn sdt_address(&self) -> usize {
        self.pointer as usize
    }
}

impl TagTrait for EFISdt32Tag {
    const ID: TagType = TagType::Efi32;

    fn dst_len(_: &TagHeader) {}
}

/// EFI system table in 64 bit mode tag.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C, align(8))]
pub struct EFISdt64Tag {
    header: TagHeader,
    pointer: u64,
}

impl EFISdt64Tag {
    /// Create a new tag to pass the EFI64 System Table pointer.
    #[must_use]
    pub fn new(pointer: u64) -> Self {
        Self {
            header: TagHeader::new(Self::ID, size_of::<Self>().try_into().unwrap()),
            pointer,
        }
    }

    /// The physical address of a x86_64 EFI system table.
    #[must_use]
    pub const fn sdt_address(&self) -> usize {
        self.pointer as usize
    }
}

impl TagTrait for EFISdt64Tag {
    const ID: TagType = TagType::Efi64;

    fn dst_len(_: &TagHeader) {}
}

/// Tag that contains the pointer to the boot loader's UEFI image handle
/// (32-bit).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C, align(8))]
pub struct EFIImageHandle32Tag {
    header: TagHeader,
    pointer: u32,
}

impl EFIImageHandle32Tag {
    /// Constructs a new tag.
    #[cfg(feature = "builder")]
    #[must_use]
    pub fn new(pointer: u32) -> Self {
        Self {
            header: TagHeader::new(Self::ID, size_of::<Self>().try_into().unwrap()),
            pointer,
        }
    }

    /// Returns the physical address of the EFI image handle.
    #[must_use]
    pub const fn image_handle(&self) -> usize {
        self.pointer as usize
    }
}

impl TagTrait for EFIImageHandle32Tag {
    const ID: TagType = TagType::Efi32Ih;

    fn dst_len(_: &TagHeader) {}
}

/// Tag that contains the pointer to the boot loader's UEFI image handle
/// (64-bit).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C, align(8))]
pub struct EFIImageHandle64Tag {
    header: TagHeader,
    pointer: u64,
}

impl EFIImageHandle64Tag {
    /// Constructs a new tag.
    #[cfg(feature = "builder")]
    #[must_use]
    pub fn new(pointer: u64) -> Self {
        Self {
            header: TagHeader::new(Self::ID, size_of::<Self>().try_into().unwrap()),
            pointer,
        }
    }

    /// Returns the physical address of the EFI image handle.
    #[must_use]
    pub const fn image_handle(&self) -> usize {
        self.pointer as usize
    }
}

impl TagTrait for EFIImageHandle64Tag {
    const ID: TagType = TagType::Efi64Ih;

    fn dst_len(_: &TagHeader) {}
}

/// EFI ExitBootServices was not called tag. This tag has no payload and is
/// just a marker.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C, align(8))]
pub struct EFIBootServicesNotExitedTag {
    header: TagHeader,
}

impl EFIBootServicesNotExitedTag {
    /// Constructs a new tag.
    #[cfg(feature = "builder")]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(feature = "builder")]
impl Default for EFIBootServicesNotExitedTag {
    fn default() -> Self {
        Self {
            header: TagHeader::new(Self::ID, size_of::<Self>().try_into().unwrap()),
        }
    }
}

impl TagTrait for EFIBootServicesNotExitedTag {
    const ID: TagType = TagType::EfiBs;

    fn dst_len(_: &TagHeader) {}
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
