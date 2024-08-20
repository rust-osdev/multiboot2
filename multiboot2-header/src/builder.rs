//! Exports a builder [`Builder`].

use crate::{
    AddressHeaderTag, ConsoleHeaderTag, EfiBootServiceHeaderTag, EntryAddressHeaderTag,
    EntryEfi32HeaderTag, EntryEfi64HeaderTag, FramebufferHeaderTag, HeaderTagISA,
    InformationRequestHeaderTag, ModuleAlignHeaderTag, Multiboot2BasicHeader, RelocatableHeaderTag,
};
use alloc::boxed::Box;
use alloc::vec::Vec;
use multiboot2_common::{new_boxed, DynSizedStructure, MaybeDynSized};

/// Builder for a Multiboot2 header information.
#[derive(Debug)]
pub struct Builder {
    arch: HeaderTagISA,
    information_request_tag: Option<Box<InformationRequestHeaderTag>>,
    address_tag: Option<AddressHeaderTag>,
    entry_tag: Option<EntryAddressHeaderTag>,
    console_tag: Option<ConsoleHeaderTag>,
    framebuffer_tag: Option<FramebufferHeaderTag>,
    module_align_tag: Option<ModuleAlignHeaderTag>,
    efi_bs_tag: Option<EfiBootServiceHeaderTag>,
    efi_32_tag: Option<EntryEfi32HeaderTag>,
    efi_64_tag: Option<EntryEfi64HeaderTag>,
    relocatable_tag: Option<RelocatableHeaderTag>,
    // TODO add support for custom tags once someone requests it.
}

impl Builder {
    /// Set the [`RelocatableHeaderTag`] tag.
    #[must_use]
    pub const fn new(arch: HeaderTagISA) -> Self {
        Self {
            arch,
            information_request_tag: None,
            address_tag: None,
            entry_tag: None,
            console_tag: None,
            framebuffer_tag: None,
            module_align_tag: None,
            efi_bs_tag: None,
            efi_32_tag: None,
            efi_64_tag: None,
            relocatable_tag: None,
        }
    }

    /// Set the [`InformationRequestHeaderTag`] tag.
    #[must_use]
    pub fn information_request_tag(
        mut self,
        information_request_tag: Box<InformationRequestHeaderTag>,
    ) -> Self {
        self.information_request_tag = Some(information_request_tag);
        self
    }

    /// Set the [`AddressHeaderTag`] tag.
    #[must_use]
    pub const fn address_tag(mut self, address_tag: AddressHeaderTag) -> Self {
        self.address_tag = Some(address_tag);
        self
    }

    /// Set the [`EntryAddressHeaderTag`] tag.
    #[must_use]
    pub const fn entry_tag(mut self, entry_tag: EntryAddressHeaderTag) -> Self {
        self.entry_tag = Some(entry_tag);
        self
    }

    /// Set the [`ConsoleHeaderTag`] tag.
    #[must_use]
    pub const fn console_tag(mut self, console_tag: ConsoleHeaderTag) -> Self {
        self.console_tag = Some(console_tag);
        self
    }

    /// Set the [`FramebufferHeaderTag`] tag.
    #[must_use]
    pub const fn framebuffer_tag(mut self, framebuffer_tag: FramebufferHeaderTag) -> Self {
        self.framebuffer_tag = Some(framebuffer_tag);
        self
    }

    /// Set the [`ModuleAlignHeaderTag`] tag.
    #[must_use]
    pub const fn module_align_tag(mut self, module_align_tag: ModuleAlignHeaderTag) -> Self {
        self.module_align_tag = Some(module_align_tag);
        self
    }

    /// Set the [`EfiBootServiceHeaderTag`] tag.
    #[must_use]
    pub const fn efi_bs_tag(mut self, efi_bs_tag: EfiBootServiceHeaderTag) -> Self {
        self.efi_bs_tag = Some(efi_bs_tag);
        self
    }

    /// Set the [`EntryEfi32HeaderTag`] tag.
    #[must_use]
    pub const fn efi_32_tag(mut self, efi_32_tag: EntryEfi32HeaderTag) -> Self {
        self.efi_32_tag = Some(efi_32_tag);
        self
    }

    /// Set the [`EntryEfi64HeaderTag`] tag.
    #[must_use]
    pub const fn efi_64_tag(mut self, efi_64_tag: EntryEfi64HeaderTag) -> Self {
        self.efi_64_tag = Some(efi_64_tag);
        self
    }

    /// Set the [`RelocatableHeaderTag`] tag.
    #[must_use]
    pub const fn relocatable_tag(mut self, relocatable_tag: RelocatableHeaderTag) -> Self {
        self.relocatable_tag = Some(relocatable_tag);
        self
    }

    /// Returns properly aligned bytes on the heap representing a valid
    /// Multiboot2 header structure.
    #[must_use]
    pub fn build(self) -> Box<DynSizedStructure<Multiboot2BasicHeader>> {
        let header = Multiboot2BasicHeader::new(self.arch, 0);
        let mut byte_refs = Vec::new();
        if let Some(tag) = self.information_request_tag.as_ref() {
            byte_refs.push(tag.as_bytes().as_ref());
        }
        if let Some(tag) = self.address_tag.as_ref() {
            byte_refs.push(tag.as_bytes().as_ref());
        }
        if let Some(tag) = self.entry_tag.as_ref() {
            byte_refs.push(tag.as_bytes().as_ref());
        }
        if let Some(tag) = self.console_tag.as_ref() {
            byte_refs.push(tag.as_bytes().as_ref());
        }
        if let Some(tag) = self.framebuffer_tag.as_ref() {
            byte_refs.push(tag.as_bytes().as_ref());
        }
        if let Some(tag) = self.module_align_tag.as_ref() {
            byte_refs.push(tag.as_bytes().as_ref());
        }
        if let Some(tag) = self.efi_bs_tag.as_ref() {
            byte_refs.push(tag.as_bytes().as_ref());
        }
        if let Some(tag) = self.efi_32_tag.as_ref() {
            byte_refs.push(tag.as_bytes().as_ref());
        }
        if let Some(tag) = self.efi_64_tag.as_ref() {
            byte_refs.push(tag.as_bytes().as_ref());
        }
        if let Some(tag) = self.relocatable_tag.as_ref() {
            byte_refs.push(tag.as_bytes().as_ref());
        }
        // TODO add support for custom tags once someone requests it.
        new_boxed(header, byte_refs.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ConsoleHeaderTagFlags::ConsoleRequired;
    use crate::HeaderTagFlag::{Optional, Required};
    use crate::RelocatableHeaderTagPreference::High;
    use crate::{MbiTagType, Multiboot2Header};

    #[test]
    fn build_and_parse() {
        let builder = Builder::new(HeaderTagISA::I386)
            .information_request_tag(InformationRequestHeaderTag::new(
                Optional,
                &[
                    MbiTagType::Cmdline.into(),
                    MbiTagType::BootLoaderName.into(),
                    MbiTagType::Module.into(),
                    MbiTagType::BasicMeminfo.into(),
                    MbiTagType::Bootdev.into(),
                    MbiTagType::Mmap.into(),
                    MbiTagType::Vbe.into(),
                    MbiTagType::Framebuffer.into(),
                    MbiTagType::ElfSections.into(),
                    MbiTagType::Apm.into(),
                    MbiTagType::Efi32.into(),
                    MbiTagType::Efi64.into(),
                    MbiTagType::Smbios.into(),
                    MbiTagType::AcpiV1.into(),
                    MbiTagType::AcpiV2.into(),
                    MbiTagType::Network.into(),
                    MbiTagType::EfiMmap.into(),
                    MbiTagType::EfiBs.into(),
                    MbiTagType::Efi32Ih.into(),
                    MbiTagType::Efi64Ih.into(),
                    MbiTagType::LoadBaseAddr.into(),
                    MbiTagType::Custom(0x1337).into(),
                ],
            ))
            .address_tag(AddressHeaderTag::new(
                Required, 0x1000, 0x2000, 0x3000, 0x4000,
            ))
            .entry_tag(EntryAddressHeaderTag::new(Required, 0x5000))
            .console_tag(ConsoleHeaderTag::new(Required, ConsoleRequired))
            .framebuffer_tag(FramebufferHeaderTag::new(Optional, 720, 1024, 8))
            .module_align_tag(ModuleAlignHeaderTag::new(Required))
            .efi_bs_tag(EfiBootServiceHeaderTag::new(Optional))
            .efi_32_tag(EntryEfi32HeaderTag::new(Required, 0x7000))
            .efi_64_tag(EntryEfi64HeaderTag::new(Required, 0x8000))
            .relocatable_tag(RelocatableHeaderTag::new(
                Required, 0x9000, 0x10000, 4096, High,
            ));

        let structure = builder.build();
        let header =
            unsafe { Multiboot2Header::load(structure.as_bytes().as_ref().as_ptr().cast()) }
                .unwrap();

        assert!(header.verify_checksum());

        for tag in header.iter() {
            dbg!(tag);
        }

        dbg!(header.arch());
        dbg!(header.checksum());
        dbg!(header.information_request_tag());
        dbg!(header.address_tag());
        dbg!(header.entry_address_tag());
        dbg!(header.console_flags_tag());
        dbg!(header.framebuffer_tag());
        dbg!(header.module_align_tag());
        dbg!(header.efi_boot_services_tag());
        dbg!(header.entry_address_efi32_tag());
        dbg!(header.entry_address_efi64_tag());
        dbg!(header.relocatable_tag());
    }
}
