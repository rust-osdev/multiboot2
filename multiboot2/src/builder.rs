//! Module for [`Builder`].

use crate::{
    BasicMemoryInfoTag, BootInformationHeader, BootLoaderNameTag, CommandLineTag,
    EFIBootServicesNotExitedTag, EFIImageHandle32Tag, EFIImageHandle64Tag, EFIMemoryMapTag,
    EFISdt32Tag, EFISdt64Tag, ElfSectionsTag, EndTag, FramebufferTag, ImageLoadPhysAddrTag,
    MemoryMapTag, ModuleTag, RsdpV1Tag, RsdpV2Tag, SmbiosTag, TagHeader, TagType, VBEInfoTag,
};
use alloc::boxed::Box;
use alloc::vec::Vec;
use multiboot2_common::{new_boxed, DynSizedStructure, MaybeDynSized};

/// Builder for a Multiboot2 header information.
// #[derive(Debug)]
#[derive(Debug)]
pub struct Builder {
    cmdline: Option<Box<CommandLineTag>>,
    bootloader: Option<Box<BootLoaderNameTag>>,
    modules: Vec<Box<ModuleTag>>,
    meminfo: Option<BasicMemoryInfoTag>,
    // missing bootdev: Option<BootDevice>
    mmap: Option<Box<MemoryMapTag>>,
    vbe: Option<VBEInfoTag>,
    framebuffer: Option<Box<FramebufferTag>>,
    elf_sections: Option<Box<ElfSectionsTag>>,
    // missing apm:
    efi32: Option<EFISdt32Tag>,
    efi64: Option<EFISdt64Tag>,
    smbios: Vec<Box<SmbiosTag>>,
    rsdpv1: Option<RsdpV1Tag>,
    rsdpv2: Option<RsdpV2Tag>,
    // missing: network
    efi_mmap: Option<Box<EFIMemoryMapTag>>,
    efi_bs: Option<EFIBootServicesNotExitedTag>,
    efi32_ih: Option<EFIImageHandle32Tag>,
    efi64_ih: Option<EFIImageHandle64Tag>,
    image_load_addr: Option<ImageLoadPhysAddrTag>,
    custom_tags: Vec<Box<DynSizedStructure<TagHeader>>>,
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder {
    /// Creates a new builder.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            cmdline: None,
            bootloader: None,
            modules: vec![],
            meminfo: None,
            mmap: None,
            vbe: None,
            framebuffer: None,
            elf_sections: None,
            efi32: None,
            efi64: None,
            smbios: vec![],
            rsdpv1: None,
            rsdpv2: None,
            efi_mmap: None,
            efi_bs: None,
            efi32_ih: None,
            efi64_ih: None,
            image_load_addr: None,
            custom_tags: vec![],
        }
    }

    /// Sets the [`CommandLineTag`] tag.
    #[must_use]
    pub fn cmdline(mut self, cmdline: Box<CommandLineTag>) -> Self {
        self.cmdline = Some(cmdline);
        self
    }

    /// Sets the [`BootLoaderNameTag`] tag.
    #[must_use]
    pub fn bootloader(mut self, bootloader: Box<BootLoaderNameTag>) -> Self {
        self.bootloader = Some(bootloader);
        self
    }

    /// Adds a [`ModuleTag`] tag.
    #[must_use]
    pub fn add_module(mut self, module: Box<ModuleTag>) -> Self {
        self.modules.push(module);
        self
    }

    /// Sets the [`BasicMemoryInfoTag`] tag.
    #[must_use]
    pub const fn meminfo(mut self, meminfo: BasicMemoryInfoTag) -> Self {
        self.meminfo = Some(meminfo);
        self
    }

    /// Sets the [`MemoryMapTag`] tag.
    #[must_use]
    pub fn mmap(mut self, mmap: Box<MemoryMapTag>) -> Self {
        self.mmap = Some(mmap);
        self
    }

    /// Sets the [`VBEInfoTag`] tag.
    #[must_use]
    pub const fn vbe(mut self, vbe: VBEInfoTag) -> Self {
        self.vbe = Some(vbe);
        self
    }

    /// Sets the [`FramebufferTag`] tag.
    #[must_use]
    pub fn framebuffer(mut self, framebuffer: Box<FramebufferTag>) -> Self {
        self.framebuffer = Some(framebuffer);
        self
    }

    /// Sets the [`ElfSectionsTag`] tag.
    #[must_use]
    pub fn elf_sections(mut self, elf_sections: Box<ElfSectionsTag>) -> Self {
        self.elf_sections = Some(elf_sections);
        self
    }

    /// Sets the [`EFISdt32Tag`] tag.
    #[must_use]
    pub const fn efi32(mut self, efi32: EFISdt32Tag) -> Self {
        self.efi32 = Some(efi32);
        self
    }

    /// Sets the [`EFISdt64Tag`] tag.
    #[must_use]
    pub const fn efi64(mut self, efi64: EFISdt64Tag) -> Self {
        self.efi64 = Some(efi64);
        self
    }

    /// Adds a [`SmbiosTag`] tag.
    #[must_use]
    pub fn add_smbios(mut self, smbios: Box<SmbiosTag>) -> Self {
        self.smbios.push(smbios);
        self
    }

    /// Sets the [`RsdpV1Tag`] tag.
    #[must_use]
    pub const fn rsdpv1(mut self, rsdpv1: RsdpV1Tag) -> Self {
        self.rsdpv1 = Some(rsdpv1);
        self
    }

    /// Sets the [`RsdpV2Tag`] tag.
    #[must_use]
    pub const fn rsdpv2(mut self, rsdpv2: RsdpV2Tag) -> Self {
        self.rsdpv2 = Some(rsdpv2);
        self
    }

    /// Sets the [`EFIMemoryMapTag`] tag.
    #[must_use]
    pub fn efi_mmap(mut self, efi_mmap: Box<EFIMemoryMapTag>) -> Self {
        self.efi_mmap = Some(efi_mmap);
        self
    }

    /// Sets the [`EFIBootServicesNotExitedTag`] tag.
    #[must_use]
    pub const fn efi_bs(mut self, efi_bs: EFIBootServicesNotExitedTag) -> Self {
        self.efi_bs = Some(efi_bs);
        self
    }

    /// Sets the [`EFIImageHandle32Tag`] tag.
    #[must_use]
    pub const fn efi32_ih(mut self, efi32_ih: EFIImageHandle32Tag) -> Self {
        self.efi32_ih = Some(efi32_ih);
        self
    }

    /// Sets the [`EFIImageHandle64Tag`] tag.
    #[must_use]
    pub const fn efi64_ih(mut self, efi64_ih: EFIImageHandle64Tag) -> Self {
        self.efi64_ih = Some(efi64_ih);
        self
    }

    /// Sets the [`ImageLoadPhysAddrTag`] tag.
    #[must_use]
    pub const fn image_load_addr(mut self, image_load_addr: ImageLoadPhysAddrTag) -> Self {
        self.image_load_addr = Some(image_load_addr);
        self
    }

    /// Adds a custom tag.
    #[must_use]
    pub fn add_custom_tag(mut self, custom_tag: Box<DynSizedStructure<TagHeader>>) -> Self {
        if let TagType::Custom(_c) = custom_tag.header().typ.into() {
            self.custom_tags.push(custom_tag);
        } else {
            panic!("Only for custom types!");
        }
        self
    }

    /// Returns properly aligned bytes on the heap representing a valid
    /// Multiboot2 header structure.
    #[must_use]
    pub fn build(self) -> Box<DynSizedStructure<BootInformationHeader>> {
        let header = BootInformationHeader::new(0);
        let mut byte_refs = Vec::new();
        if let Some(tag) = self.cmdline.as_ref() {
            byte_refs.push(tag.as_bytes().as_ref());
        }
        if let Some(tag) = self.bootloader.as_ref() {
            byte_refs.push(tag.as_bytes().as_ref());
        }
        for i in &self.modules {
            byte_refs.push(i.as_bytes().as_ref());
        }
        if let Some(tag) = self.meminfo.as_ref() {
            byte_refs.push(tag.as_bytes().as_ref());
        }
        if let Some(tag) = self.mmap.as_ref() {
            byte_refs.push(tag.as_bytes().as_ref());
        }
        if let Some(tag) = self.vbe.as_ref() {
            byte_refs.push(tag.as_bytes().as_ref());
        }
        if let Some(tag) = self.framebuffer.as_ref() {
            byte_refs.push(tag.as_bytes().as_ref());
        }
        if let Some(tag) = self.elf_sections.as_ref() {
            byte_refs.push(tag.as_bytes().as_ref());
        }
        if let Some(tag) = self.efi32.as_ref() {
            byte_refs.push(tag.as_bytes().as_ref());
        }
        if let Some(tag) = self.efi64.as_ref() {
            byte_refs.push(tag.as_bytes().as_ref());
        }
        for i in &self.smbios {
            byte_refs.push(i.as_bytes().as_ref());
        }
        if let Some(tag) = self.rsdpv1.as_ref() {
            byte_refs.push(tag.as_bytes().as_ref());
        }
        if let Some(tag) = self.rsdpv2.as_ref() {
            byte_refs.push(tag.as_bytes().as_ref());
        }
        if let Some(tag) = self.efi_mmap.as_ref() {
            byte_refs.push(tag.as_bytes().as_ref());
        }
        if let Some(tag) = self.efi_bs.as_ref() {
            byte_refs.push(tag.as_bytes().as_ref());
        }
        if let Some(tag) = self.efi32_ih.as_ref() {
            byte_refs.push(tag.as_bytes().as_ref());
        }
        if let Some(tag) = self.efi64_ih.as_ref() {
            byte_refs.push(tag.as_bytes().as_ref());
        }
        if let Some(tag) = self.image_load_addr.as_ref() {
            byte_refs.push(tag.as_bytes().as_ref());
        }
        for i in &self.custom_tags {
            byte_refs.push(i.as_bytes().as_ref());
        }
        let end_tag = EndTag::default();
        byte_refs.push(end_tag.as_bytes().as_ref());
        new_boxed(header, byte_refs.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framebuffer::FramebufferTypeId;
    use crate::{BootInformation, MemoryArea, MemoryAreaType, VBEControlInfo, VBEModeInfo};
    use uefi_raw::table::boot::MemoryDescriptor;

    #[test]
    fn build_and_parse() {
        let builder = Builder::new()
            .cmdline(CommandLineTag::new("this is a command line"))
            .bootloader(BootLoaderNameTag::new("this is the bootloader"))
            .add_module(ModuleTag::new(0x1000, 0x2000, "module 1"))
            .add_module(ModuleTag::new(0x3000, 0x4000, "module 2"))
            .meminfo(BasicMemoryInfoTag::new(0x4000, 0x5000))
            .mmap(MemoryMapTag::new(&[MemoryArea::new(
                0x1000000,
                0x1000,
                MemoryAreaType::Available,
            )]))
            .vbe(VBEInfoTag::new(
                42,
                2,
                4,
                9,
                VBEControlInfo::default(),
                VBEModeInfo::default(),
            ))
            // Currently causes UB.
            .framebuffer(FramebufferTag::new(
                0x1000,
                1,
                756,
                1024,
                8,
                FramebufferTypeId::Text,
            ))
            .elf_sections(ElfSectionsTag::new(0, 32, 0, &[]))
            .efi32(EFISdt32Tag::new(0x1000))
            .efi64(EFISdt64Tag::new(0x1000))
            .add_smbios(SmbiosTag::new(0, 0, &[1, 2, 3]))
            .add_smbios(SmbiosTag::new(1, 1, &[4, 5, 6]))
            .rsdpv1(RsdpV1Tag::new(0, *b"abcdef", 5, 6))
            .rsdpv2(RsdpV2Tag::new(0, *b"abcdef", 5, 6, 5, 4, 7))
            .efi_mmap(EFIMemoryMapTag::new_from_descs(&[
                MemoryDescriptor::default(),
                MemoryDescriptor::default(),
            ]))
            .efi_bs(EFIBootServicesNotExitedTag::new())
            .efi32_ih(EFIImageHandle32Tag::new(0x1000))
            .efi64_ih(EFIImageHandle64Tag::new(0x1000))
            .image_load_addr(ImageLoadPhysAddrTag::new(0x1000))
            .add_custom_tag(new_boxed::<DynSizedStructure<TagHeader>>(
                TagHeader::new(TagType::Custom(0x1337), 0),
                &[],
            ));

        let structure = builder.build();

        let info = unsafe { BootInformation::load(structure.as_bytes().as_ptr().cast()) }.unwrap();
        for tag in info.tags() {
            // Mainly a test for Miri.
            dbg!(tag.header(), tag.payload().len());
        }
    }
}
