//! All tags related to (U)EFI with the exception of EFI memory tags:
//!
//! - [`EFISdt32Tag`]
//! - [`EFISdt64Tag`]
//! - [`EFIImageHandle32Tag`]
//! - [`EFIImageHandle64Tag`]
//! - [`EFIBootServicesNotExitedTag`]

use crate::tag::TagHeader;
use crate::TagType;
use core::mem::size_of;
use multiboot2_common::{MaybeDynSized, Tag};

/// EFI system table in 32 bit mode tag.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C, align(8))]
pub struct EFISdt32Tag {
    header: TagHeader,
    pointer: u32,
}

impl EFISdt32Tag {
    const BASE_SIZE: usize = size_of::<TagHeader>() + size_of::<u32>();

    /// Create a new tag to pass the EFI32 System Table pointer.
    #[must_use]
    pub fn new(pointer: u32) -> Self {
        Self {
            header: TagHeader::new(Self::ID, Self::BASE_SIZE as u32),
            pointer,
        }
    }

    /// The physical address of a i386 EFI system table.
    #[must_use]
    pub const fn sdt_address(&self) -> usize {
        self.pointer as usize
    }
}

impl MaybeDynSized for EFISdt32Tag {
    type Header = TagHeader;

    const BASE_SIZE: usize = size_of::<Self>();

    fn dst_len(_: &TagHeader) {}
}

impl Tag for EFISdt32Tag {
    type IDType = TagType;

    const ID: TagType = TagType::Efi32;
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

impl MaybeDynSized for EFISdt64Tag {
    type Header = TagHeader;

    const BASE_SIZE: usize = size_of::<Self>();

    fn dst_len(_: &TagHeader) {}
}

impl Tag for EFISdt64Tag {
    type IDType = TagType;

    const ID: TagType = TagType::Efi64;
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
    const BASE_SIZE: usize = size_of::<TagHeader>() + size_of::<u32>();

    /// Constructs a new tag.
    #[must_use]
    pub fn new(pointer: u32) -> Self {
        Self {
            header: TagHeader::new(Self::ID, Self::BASE_SIZE as u32),
            pointer,
        }
    }

    /// Returns the physical address of the EFI image handle.
    #[must_use]
    pub const fn image_handle(&self) -> usize {
        self.pointer as usize
    }
}

impl MaybeDynSized for EFIImageHandle32Tag {
    type Header = TagHeader;

    const BASE_SIZE: usize = size_of::<Self>();

    fn dst_len(_: &TagHeader) {}
}

impl Tag for EFIImageHandle32Tag {
    type IDType = TagType;

    const ID: TagType = TagType::Efi32Ih;
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

impl MaybeDynSized for EFIImageHandle64Tag {
    type Header = TagHeader;

    const BASE_SIZE: usize = size_of::<Self>();

    fn dst_len(_: &TagHeader) {}
}

impl Tag for EFIImageHandle64Tag {
    type IDType = TagType;

    const ID: TagType = TagType::Efi64Ih;
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
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for EFIBootServicesNotExitedTag {
    fn default() -> Self {
        Self {
            header: TagHeader::new(Self::ID, size_of::<Self>().try_into().unwrap()),
        }
    }
}

impl MaybeDynSized for EFIBootServicesNotExitedTag {
    type Header = TagHeader;

    const BASE_SIZE: usize = size_of::<Self>();

    fn dst_len(_: &TagHeader) {}
}

impl Tag for EFIBootServicesNotExitedTag {
    type IDType = TagType;

    const ID: TagType = TagType::EfiBs;
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
