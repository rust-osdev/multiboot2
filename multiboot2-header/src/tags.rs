//! Definition for all types of "Multiboot2 header tags". The values are taken from the example C
//! code at the end of the official Multiboot2 spec. These tags follow in memory right after
//! [`crate::Multiboot2BasicHeader`].

use core::mem;
use multiboot2_common::Header;

/// ISA/ARCH in Multiboot2 header.
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HeaderTagISA {
    /// Spec: "means 32-bit (protected) mode of i386".
    /// Caution: This is confusing. If you use the EFI64-tag
    /// on an UEFI system, the machine will boot into `64-bit long mode`.
    /// Therefore this tag should be understood as "arch=x86|x86_64".
    I386 = 0,
    /// 32-bit MIPS
    MIPS32 = 4,
}

/// Possible types for header tags of a Multiboot2 header. The names and values are taken
/// from the example C code at the bottom of the Multiboot2 specification. This value
/// stands in the `typ` property of [`HeaderTagHeader`].
#[repr(u16)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HeaderTagType {
    /// Type for [`crate::EndHeaderTag`].
    End = 0,
    /// Type for [`crate::InformationRequestHeaderTag`].
    InformationRequest = 1,
    /// Type for [`crate::AddressHeaderTag`].
    Address = 2,
    /// Type for [`crate::EntryAddressHeaderTag`].
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

impl HeaderTagType {
    /// Returns the number of possible variants.
    #[must_use]
    pub const fn count() -> u32 {
        11
    }
}

/// Flags for Multiboot2 header tags.
#[repr(u16)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HeaderTagFlag {
    /// Bootloader must provide this tag. If this is not possible, the
    /// bootloader will fail loading the kernel.
    Required = 0,
    /// Bootloader should provide the tag if possible.
    Optional = 1,
}

/// The common header that all header tags share. Specific tags may have
/// additional fields that depend on the `typ` and the `size` field.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct HeaderTagHeader {
    typ: HeaderTagType, /* u16 */
    // u16 value
    flags: HeaderTagFlag, /* u16 */
    size: u32,
    // Followed by optional additional tag specific fields.
}

impl HeaderTagHeader {
    /// Creates a new header.
    #[must_use]
    pub const fn new(typ: HeaderTagType, flags: HeaderTagFlag, size: u32) -> Self {
        Self { typ, flags, size }
    }

    /// Returns the [`HeaderTagType`].
    #[must_use]
    pub const fn typ(&self) -> HeaderTagType {
        self.typ
    }

    /// Returns the [`HeaderTagFlag`]s.
    #[must_use]
    pub const fn flags(&self) -> HeaderTagFlag {
        self.flags
    }

    /// Returns the size.
    #[must_use]
    pub const fn size(&self) -> u32 {
        self.size
    }
}

impl Header for HeaderTagHeader {
    fn payload_len(&self) -> usize {
        self.size as usize - mem::size_of::<Self>()
    }

    fn set_size(&mut self, total_size: usize) {
        self.size = total_size as u32;
    }
}

#[cfg(test)]
mod tests {
    use crate::HeaderTagHeader;

    #[test]
    fn test_assert_size() {
        assert_eq!(core::mem::size_of::<HeaderTagHeader>(), 2 + 2 + 4);
    }
}
