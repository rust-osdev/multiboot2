//! Definition for all types of "Multiboot2 header tags". The values are taken from the example C
//! code at the end of the official Multiboot2 spec. These tags follow in memory right after
//! [`crate::Multiboot2BasicHeader`].

use core::fmt::Debug;

/// ISA/ARCH in Multiboot2 header.
#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum HeaderTagISA {
    /// Spec: "means 32-bit (protected) mode of i386".
    /// Caution: This is confusing. If you use the EFI64-tag
    /// on an UEFI system, you get into `64-bit long mode`.
    /// Therefore this tag should be understood as "arch=x86|x86_64".
    I386 = 0,
    /// 32-bit MIPS
    MIPS32 = 4,
}

/// Possible types for header tags of a Multiboot2 header. The names and values are taken
/// from the example C code at the bottom of the Multiboot2 specification. This value
/// stands in the `typ` property of [`crate::tags::HeaderTag`].
#[repr(u16)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum HeaderTagType {
    /// Type for [`crate::EndHeaderTag`].
    End = 0,
    /// Type for [`crate::InformationRequestHeaderTag`].
    InformationRequest = 1,
    /// Type for [`crate::AddressHeaderTag`].
    Address = 2,
    /// Type for [`crate::EntryHeaderTag`].
    EntryAddress = 3,
    /// Type for [`crate::ConsoleHeaderTag`].
    ConsoleFlags = 4,
    /// Type for [`crate::FramebufferHeaderTag`].
    Framebuffer = 5,
    /// Type for [`crate::ModuleAlignHeaderTag`].
    ModuleAlign = 6,
    /// Type for [`crate::EfiBootServiceHeaderTag`].
    EfiBS = 7,
    /// Type for [`crate::EntryEfi32HeaderTag`].
    EntryAddressEFI32 = 8,
    /// Type for [`crate::EntryEfi64HeaderTag`].
    EntryAddressEFI64 = 9,
    /// Type for [`crate::RelocatableHeaderTag`].
    Relocatable = 10,
}

/// Flags for multiboot2 header tags.
#[repr(u16)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum HeaderTagFlag {
    Required = 0,
    Optional = 1,
}

/// Common properties for all header tags. Other tags may have additional fields
/// that depend on the `typ` and the `size` field.
#[derive(Copy, Clone, Debug)]
#[repr(C, packed(8))]
pub struct HeaderTag {
    // u16 value
    typ: HeaderTagType,
    // u16 value
    flags: HeaderTagFlag,
    size: u32,
    // maybe additional fields (tag specific)
}

impl HeaderTag {
    pub const fn typ(&self) -> HeaderTagType {
        self.typ
    }
    pub const fn flags(&self) -> HeaderTagFlag {
        self.flags
    }
    pub const fn size(&self) -> u32 {
        self.size
    }
}
