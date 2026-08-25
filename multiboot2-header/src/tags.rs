//! Definition for all types of "Multiboot2 header tags". The values are taken from the example C
//! code at the end of the official Multiboot2 spec. These tags follow in memory right after
//! [`crate::Multiboot2BasicHeader`].

use multiboot2_common::Header;

multiboot2_common::raw_type! {
    /// Serialized form of [`HeaderTagISA`] that matches the binary
    /// representation (`u32`). This value stands in the `arch` property of
    /// [`crate::Multiboot2BasicHeader`].
    pub struct HeaderTagISARaw(u32);

    /// ISA/ARCH in Multiboot2 header.
    pub enum HeaderTagISA {
        /// Spec: "means 32-bit (protected) mode of i386".
        /// Caution: This is confusing. If you use the EFI64-tag
        /// on an UEFI system, the machine will boot into `64-bit long mode`.
        /// Therefore this tag should be understood as "arch=x86|x86_64".
        I386 = 0,
        /// 32-bit MIPS
        MIPS32 = 4,
    }
}

multiboot2_common::raw_type! {
    /// Serialized form of [`HeaderTagType`] that matches the binary
    /// representation (`u16`). This value stands in the `typ` property of
    /// [`HeaderTagHeader`].
    pub struct HeaderTagTypeRaw(u16);

    /// Possible types for header tags of a Multiboot2 header.
    ///
    /// The names and values are taken from the example C code at the bottom of
    /// the Multiboot2 specification.
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
}

multiboot2_common::raw_type! {
    /// Serialized form of [`HeaderTagFlag`] that matches the binary
    /// representation (`u16`). This value stands in the `flags` property of
    /// [`HeaderTagHeader`].
    pub struct HeaderTagFlagRaw(u16);

    /// Flags for Multiboot2 header tags.
    pub enum HeaderTagFlag {
        /// The bootloader must provide this tag. If this is not possible, the
        /// bootloader will fail to load the kernel.
        Required = 0,
        /// The bootloader should provide the tag if possible.
        Optional = 1,
    }
}

/// The common header that all header tags share. Specific tags may have
/// additional fields that depend on the `typ` and the `size` field.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C, align(8))]
pub struct HeaderTagHeader {
    typ: HeaderTagTypeRaw,   /* u16 */
    flags: HeaderTagFlagRaw, /* u16 */
    size: u32,
    // Followed by optional additional tag-specific fields.
}

impl HeaderTagHeader {
    /// Creates a new header.
    #[must_use]
    pub const fn new(typ: HeaderTagType, flags: HeaderTagFlag, size: u32) -> Self {
        Self {
            typ: HeaderTagTypeRaw::new(typ.val()),
            flags: HeaderTagFlagRaw::new(flags.val()),
            size,
        }
    }

    /// Returns the [`HeaderTagType`].
    #[must_use]
    pub const fn typ(&self) -> HeaderTagType {
        HeaderTagType::from_val(self.typ.get())
    }

    /// Returns the [`HeaderTagFlag`]s.
    #[must_use]
    pub const fn flags(&self) -> HeaderTagFlag {
        HeaderTagFlag::from_val(self.flags.get())
    }

    /// Returns the size.
    #[must_use]
    pub const fn size(&self) -> u32 {
        self.size
    }
}

impl Header for HeaderTagHeader {
    fn total_size(&self) -> usize {
        self.size as usize
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
        assert_eq!(size_of::<HeaderTagHeader>(), 2 + 2 + 4);
    }
}
