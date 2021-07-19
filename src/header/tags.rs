//! Type definitions for "multiboot2 header tags". These tags occur in a binary if it
//! is multiboot2-compliant, for example a kernel.
//!
//! The values are taken from the example C code at the end of the official multiboot2 spec.
//!
//! This type definitions are only beneficial to parse such a structure. They can't be used
//! to construct a multiboot2 header as a static global variable. To write a multiboot2
//! header, it is highly recommended to do this directly in assembly, because of the
//! unknown size of structs or some addresses are not known yet (keyword: relocations).

use core::fmt::Debug;

/// ISA/ARCH in multiboot2 header.
#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum HeaderTagISA {
    /// Spec: "means 32-bit (protected) mode of i386".
    /// Caution: This is confusing. If you use the EFI64-tag
    /// on an UEFI system, you get into `64-bit long mode`.
    /// Therefore this tag should be understood as "arch=x86".
    I386 = 0,
    /// 32-bit MIPS
    MIPS32 = 4,
}

/// Possible types for header tags of a multiboot2 header. The names and values are taken
/// from the example C code at the bottom of the Multiboot2 specification.
#[repr(u16)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum HeaderTagType {
    /// Type for [`EndHeaderTag`].
    End = 0,
    /// Type for [`InformationRequestHeaderTag`].
    InformationRequest = 1,
    /// Type for [`AddressHeaderTag`].
    Address = 2,
    /// Type for [`EntryHeaderTag`].
    EntryAddress = 3,
    /// Type for [`ConsoleHeaderTag`].
    ConsoleFlags = 4,
    /// Type for [`FramebufferHeaderTag`].
    Framebuffer = 5,
    /// Type for [`ModuleAlignHeaderTag`].
    ModuleAlign = 6,
    /// Type for [`EfiBootServiceHeaderTag`].
    EfiBS = 7,
    /// Type for [`EntryEfi32HeaderTag`].
    EntryAddressEFI32 = 8,
    /// Type for [`EntryEfi64HeaderTag`].
    EntryAddressEFI64 = 9,
    /// Type for [`RelocatableHeaderTag`].
    Relocatable = 10,
}

/// Flags for multiboot2 header tags.
#[repr(u16)]
#[derive(Copy, Clone, Debug)]
pub enum HeaderTagFlag {
    Required = 0,
    Optional = 1,
}

/// Common structure for all header tags.
#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct HeaderTag {
    // u16 value
    typ: HeaderTagType,
    // u16 value
    flags: HeaderTagFlag,
    size: u32,
    // maybe additional fields (tag specific)
}

impl HeaderTag {
    // never needed to construct this publicly
    /*
    pub fn new(typ: HeaderTagType, flags: HeaderTagFlag, size: u32) -> Self {
        HeaderTag { typ, flags, size }
    }*/

    pub fn typ(&self) -> HeaderTagType {
        self.typ
    }
    pub fn flags(&self) -> HeaderTagFlag {
        self.flags
    }
    pub fn size(&self) -> u32 {
        self.size
    }
}
