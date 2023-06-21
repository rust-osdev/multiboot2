//! Exports item [`InformationBuilder`].
use crate::builder::traits::StructAsBytes;
use crate::{
    BasicMemoryInfoTag, BootInformationInner, BootLoaderNameTag, CommandLineTag,
    EFIBootServicesNotExited, EFIImageHandle32, EFIImageHandle64, EFIMemoryMapTag, EFISdt32,
    EFISdt64, ElfSectionsTag, EndTag, FramebufferTag, ImageLoadPhysAddrTag, MemoryMapTag, ModuleTag,
    RsdpV1Tag, RsdpV2Tag, SmbiosTag,
};

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::mem::size_of;

/// Builder to construct a valid Multiboot2 information dynamically at runtime.
/// The tags will appear in the order of their corresponding enumeration,
/// except for the END tag.
#[derive(Debug, PartialEq, Eq)]
pub struct InformationBuilder {
    basic_memory_info_tag: Option<BasicMemoryInfoTag>,
    boot_loader_name_tag: Option<Box<BootLoaderNameTag>>,
    command_line_tag: Option<Box<CommandLineTag>>,
    efi_boot_services_not_exited: Option<EFIBootServicesNotExited>,
    efi_image_handle32: Option<EFIImageHandle32>,
    efi_image_handle64: Option<EFIImageHandle64>,
    efi_memory_map_tag: Option<Box<EFIMemoryMapTag>>,
    elf_sections_tag: Option<Box<ElfSectionsTag>>,
    framebuffer_tag: Option<Box<FramebufferTag>>,
    image_load_addr: Option<ImageLoadPhysAddrTag>,
    memory_map_tag: Option<Box<MemoryMapTag>>,
    module_tags: Vec<Box<ModuleTag>>,
    efisdt32: Option<EFISdt32>,
    efisdt64: Option<EFISdt64>,
    rsdp_v1_tag: Option<RsdpV1Tag>,
    rsdp_v2_tag: Option<RsdpV2Tag>,
    smbios_tags: Vec<Box<SmbiosTag>>,
}

impl InformationBuilder {
    pub const fn new() -> Self {
        Self {
            basic_memory_info_tag: None,
            boot_loader_name_tag: None,
            command_line_tag: None,
            efisdt32: None,
            efisdt64: None,
            efi_boot_services_not_exited: None,
            efi_image_handle32: None,
            efi_image_handle64: None,
            efi_memory_map_tag: None,
            elf_sections_tag: None,
            framebuffer_tag: None,
            image_load_addr: None,
            memory_map_tag: None,
            module_tags: Vec::new(),
            rsdp_v1_tag: None,
            rsdp_v2_tag: None,
            smbios_tags: Vec::new(),
        }
    }

    /// Returns the size, if the value is a multiple of 8 or returns
    /// the next number that is a multiple of 8. With this, one can
    /// easily calculate the size of a Multiboot2 header, where
    /// all the tags are 8-byte aligned.
    const fn size_or_up_aligned(size: usize) -> usize {
        let remainder = size % 8;
        if remainder == 0 {
            size
        } else {
            size + 8 - remainder
        }
    }

    /// Returns the expected length of the Multiboot2 header,
    /// when the `build()`-method gets called.
    pub fn expected_len(&self) -> usize {
        let base_len = size_of::<BootInformationInner>();
        // size_or_up_aligned not required, because length is 16 and the
        // begin is 8 byte aligned => first tag automatically 8 byte aligned
        let mut len = Self::size_or_up_aligned(base_len);
        if let Some(tag) = &self.basic_memory_info_tag {
            // we use size_or_up_aligned, because each tag will start at an 8 byte aligned address
            len += Self::size_or_up_aligned(tag.byte_size())
        }
        if let Some(tag) = &self.boot_loader_name_tag {
            len += Self::size_or_up_aligned(tag.byte_size())
        }
        if let Some(tag) = &self.command_line_tag {
            len += Self::size_or_up_aligned(tag.byte_size())
        }
        if let Some(tag) = &self.efisdt32 {
            len += Self::size_or_up_aligned(tag.byte_size())
        }
        if let Some(tag) = &self.efisdt64 {
            len += Self::size_or_up_aligned(tag.byte_size())
        }
        if let Some(tag) = &self.efi_boot_services_not_exited {
            len += Self::size_or_up_aligned(tag.byte_size())
        }
        if let Some(tag) = &self.efi_image_handle32 {
            len += Self::size_or_up_aligned(tag.byte_size())
        }
        if let Some(tag) = &self.efi_image_handle64 {
            len += Self::size_or_up_aligned(tag.byte_size())
        }
        if let Some(tag) = &self.efi_memory_map_tag {
            len += Self::size_or_up_aligned(tag.byte_size())
        }
        if let Some(tag) = &self.elf_sections_tag {
            len += Self::size_or_up_aligned(tag.byte_size())
        }
        if let Some(tag) = &self.framebuffer_tag {
            len += Self::size_or_up_aligned(tag.byte_size())
        }
        if let Some(tag) = &self.image_load_addr {
            len += Self::size_or_up_aligned(tag.byte_size())
        }
        if let Some(tag) = &self.memory_map_tag {
            len += Self::size_or_up_aligned(tag.byte_size())
        }
        for tag in &self.module_tags {
            len += Self::size_or_up_aligned(tag.byte_size())
        }
        if let Some(tag) = &self.rsdp_v1_tag {
            len += Self::size_or_up_aligned(tag.byte_size())
        }
        if let Some(tag) = &self.rsdp_v2_tag {
            len += Self::size_or_up_aligned(tag.byte_size())
        }
        for tag in &self.smbios_tags {
            len += Self::size_or_up_aligned(tag.byte_size())
        }
        // only here size_or_up_aligned is not important, because it is the last tag
        len += size_of::<EndTag>();
        len
    }

    /// Adds the bytes of a tag to the final Multiboot2 information byte vector.
    /// Align should be true for all tags except the end tag.
    fn build_add_bytes(dest: &mut Vec<u8>, source: &[u8], is_end_tag: bool) {
        dest.extend(source);
        if !is_end_tag {
            let size = source.len();
            let size_to_8_align = Self::size_or_up_aligned(size);
            let size_to_8_align_diff = size_to_8_align - size;
            // fill zeroes so that next data block is 8-byte aligned
            dest.extend([0].repeat(size_to_8_align_diff));
        }
    }

    /// Constructs the bytes for a valid Multiboot2 information with the given properties.
    /// The bytes can be casted to a Multiboot2 structure.
    pub fn build(self) -> Vec<u8> {
        let mut data = Vec::new();

        Self::build_add_bytes(
            &mut data,
            // important that we write the correct expected length into the header!
            &BootInformationInner::new(self.expected_len() as u32).struct_as_bytes(),
            false,
        );

        if let Some(tag) = self.basic_memory_info_tag.as_ref() {
            Self::build_add_bytes(&mut data, &tag.struct_as_bytes(), false)
        }
        if let Some(tag) = self.boot_loader_name_tag.as_ref() {
            Self::build_add_bytes(&mut data, &tag.struct_as_bytes(), false)
        }
        if let Some(tag) = self.command_line_tag.as_ref() {
            Self::build_add_bytes(&mut data, &tag.struct_as_bytes(), false)
        }
        if let Some(tag) = self.efisdt32.as_ref() {
            Self::build_add_bytes(&mut data, &tag.struct_as_bytes(), false)
        }
        if let Some(tag) = self.efisdt64.as_ref() {
            Self::build_add_bytes(&mut data, &tag.struct_as_bytes(), false)
        }
        if let Some(tag) = self.efi_boot_services_not_exited.as_ref() {
            Self::build_add_bytes(&mut data, &tag.struct_as_bytes(), false)
        }
        if let Some(tag) = self.efi_image_handle32.as_ref() {
            Self::build_add_bytes(&mut data, &tag.struct_as_bytes(), false)
        }
        if let Some(tag) = self.efi_image_handle64.as_ref() {
            Self::build_add_bytes(&mut data, &tag.struct_as_bytes(), false)
        }
        if let Some(tag) = self.efi_memory_map_tag.as_ref() {
            Self::build_add_bytes(&mut data, &tag.struct_as_bytes(), false)
        }
        if let Some(tag) = self.elf_sections_tag.as_ref() {
            Self::build_add_bytes(&mut data, &tag.struct_as_bytes(), false)
        }
        if let Some(tag) = self.framebuffer_tag.as_ref() {
            Self::build_add_bytes(&mut data, &tag.struct_as_bytes(), false)
        }
        if let Some(tag) = self.image_load_addr.as_ref() {
            Self::build_add_bytes(&mut data, &tag.struct_as_bytes(), false)
        }
        if let Some(tag) = self.memory_map_tag.as_ref() {
            Self::build_add_bytes(&mut data, &tag.struct_as_bytes(), false)
        }
        for tag in self.module_tags {
            Self::build_add_bytes(&mut data, &tag.struct_as_bytes(), false)
        }
        if let Some(tag) = self.rsdp_v1_tag.as_ref() {
            Self::build_add_bytes(&mut data, &tag.struct_as_bytes(), false)
        }
        if let Some(tag) = self.rsdp_v2_tag.as_ref() {
            Self::build_add_bytes(&mut data, &tag.struct_as_bytes(), false)
        }
        for tag in self.smbios_tags {
            Self::build_add_bytes(&mut data, &tag.struct_as_bytes(), false)
        }

        Self::build_add_bytes(&mut data, &EndTag::default().struct_as_bytes(), true);

        data
    }

    pub fn basic_memory_info_tag(&mut self, basic_memory_info_tag: BasicMemoryInfoTag) {
        self.basic_memory_info_tag = Some(basic_memory_info_tag)
    }

    pub fn bootloader_name_tag(&mut self, boot_loader_name_tag: Box<BootLoaderNameTag>) {
        self.boot_loader_name_tag = Some(boot_loader_name_tag);
    }

    pub fn command_line_tag(&mut self, command_line_tag: Box<CommandLineTag>) {
        self.command_line_tag = Some(command_line_tag);
    }

    pub fn efisdt32(&mut self, efisdt32: EFISdt32) {
        self.efisdt32 = Some(efisdt32);
    }

    pub fn efisdt64(&mut self, efisdt64: EFISdt64) {
        self.efisdt64 = Some(efisdt64);
    }

    pub fn efi_boot_services_not_exited(&mut self) {
        self.efi_boot_services_not_exited = Some(EFIBootServicesNotExited::new());
    }

    pub fn efi_image_handle32(&mut self, efi_image_handle32: EFIImageHandle32) {
        self.efi_image_handle32 = Some(efi_image_handle32);
    }

    pub fn efi_image_handle64(&mut self, efi_image_handle64: EFIImageHandle64) {
        self.efi_image_handle64 = Some(efi_image_handle64);
    }

    pub fn efi_memory_map_tag(&mut self, efi_memory_map_tag: Box<EFIMemoryMapTag>) {
        self.efi_memory_map_tag = Some(efi_memory_map_tag);
    }

    pub fn elf_sections_tag(&mut self, elf_sections_tag: Box<ElfSectionsTag>) {
        self.elf_sections_tag = Some(elf_sections_tag);
    }

    pub fn framebuffer_tag(&mut self, framebuffer_tag: Box<FramebufferTag>) {
        self.framebuffer_tag = Some(framebuffer_tag);
    }

    pub fn image_load_addr(&mut self, image_load_addr: ImageLoadPhysAddrTag) {
        self.image_load_addr = Some(image_load_addr);
    }

    pub fn memory_map_tag(&mut self, memory_map_tag: Box<MemoryMapTag>) {
        self.memory_map_tag = Some(memory_map_tag);
    }

    pub fn add_module_tag(&mut self, module_tag: Box<ModuleTag>) {
        self.module_tags.push(module_tag);
    }

    pub fn rsdp_v1_tag(&mut self, rsdp_v1_tag: RsdpV1Tag) {
        self.rsdp_v1_tag = Some(rsdp_v1_tag);
    }

    pub fn rsdp_v2_tag(&mut self, rsdp_v2_tag: RsdpV2Tag) {
        self.rsdp_v2_tag = Some(rsdp_v2_tag);
    }

    pub fn add_smbios_tag(&mut self, smbios_tag: Box<SmbiosTag>) {
        self.smbios_tags.push(smbios_tag);
    }
}

#[cfg(test)]
mod tests {
    use crate::builder::information::InformationBuilder;
    use crate::{load, BasicMemoryInfoTag, CommandLineTag, ModuleTag};

    #[test]
    fn test_size_or_up_aligned() {
        assert_eq!(0, InformationBuilder::size_or_up_aligned(0));
        assert_eq!(8, InformationBuilder::size_or_up_aligned(1));
        assert_eq!(8, InformationBuilder::size_or_up_aligned(8));
        assert_eq!(16, InformationBuilder::size_or_up_aligned(9));
    }

    #[test]
    fn test_builder() {
        let mut builder = InformationBuilder::new();
        // Multiboot2 basic information + end tag
        let mut expected_len = 8 + 8;
        assert_eq!(builder.expected_len(), expected_len);

        // the most simple tag
        builder.basic_memory_info_tag(BasicMemoryInfoTag::new(640, 7 * 1024));
        expected_len += 16;
        assert_eq!(builder.expected_len(), expected_len);
        // a tag that has a dynamic size
        builder.command_line_tag(CommandLineTag::new("test"));
        expected_len += 8 + 5 + 3; // padding
        assert_eq!(builder.expected_len(), expected_len);
        // many modules
        builder.add_module_tag(ModuleTag::new(0, 1234, "module1"));
        expected_len += 16 + 8;
        assert_eq!(builder.expected_len(), expected_len);
        builder.add_module_tag(ModuleTag::new(5678, 6789, "module2"));
        expected_len += 16 + 8;
        assert_eq!(builder.expected_len(), expected_len);

        println!("builder: {:#?}", builder);
        println!("expected_len: {} bytes", builder.expected_len());
        assert_eq!(builder.expected_len(), expected_len);

        let mb2i_data = builder.build();
        let mb2i_addr = mb2i_data.as_ptr() as usize;
        let mb2i = unsafe { load(mb2i_addr) }.expect("the generated information to be readable");
        println!("{:#?}", mb2i);
        assert_eq!(mb2i.basic_memory_info_tag().unwrap().memory_lower(), 640);
        assert_eq!(
            mb2i.basic_memory_info_tag().unwrap().memory_upper(),
            7 * 1024
        );
        assert_eq!(
            mb2i.command_line_tag().unwrap().command_line().unwrap(),
            "test"
        );
        let mut modules = mb2i.module_tags();
        let module_1 = modules.next().unwrap();
        assert_eq!(module_1.start_address(), 0);
        assert_eq!(module_1.end_address(), 1234);
        assert_eq!(module_1.cmdline().unwrap(), "module1");
        let module_2 = modules.next().unwrap();
        assert_eq!(module_2.start_address(), 5678);
        assert_eq!(module_2.end_address(), 6789);
        assert_eq!(module_2.cmdline().unwrap(), "module2");
        assert!(modules.next().is_none());
    }
}
