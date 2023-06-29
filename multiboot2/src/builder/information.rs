//! Exports item [`InformationBuilder`].
use crate::builder::traits::StructAsBytes;
use crate::{
    BasicMemoryInfoTag, BootInformationHeader, BootLoaderNameTag, CommandLineTag,
    EFIBootServicesNotExitedTag, EFIImageHandle32Tag, EFIImageHandle64Tag, EFIMemoryMapTag,
    EFISdt32Tag, EFISdt64Tag, ElfSectionsTag, EndTag, FramebufferTag, ImageLoadPhysAddrTag,
    MemoryMapTag, ModuleTag, RsdpV1Tag, RsdpV2Tag, SmbiosTag,
};

use crate::builder::BoxedDst;
use alloc::vec::Vec;
use core::mem::size_of;
use core::ops::Deref;

/// Holds the raw bytes of a boot information built with [`InformationBuilder`]
/// on the heap. The bytes returned by [`BootInformationBytes::as_bytes`] are
/// guaranteed to be properly aligned.
#[derive(Clone, Debug)]
pub struct BootInformationBytes {
    // Offset into the bytes where the MBI starts. This is necessary to
    // guarantee alignment at the moment.
    offset: usize,
    structure_len: usize,
    bytes: Vec<u8>,
}

impl BootInformationBytes {
    /// Returns the bytes. They are guaranteed to be correctly aligned.
    pub fn as_bytes(&self) -> &[u8] {
        let slice = &self.bytes[self.offset..self.offset + self.structure_len];
        // At this point, the alignment is guaranteed. If not, something is
        // broken fundamentally.
        assert_eq!(slice.as_ptr().align_offset(8), 0);
        slice
    }
}

impl Deref for BootInformationBytes {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_bytes()
    }
}

/// Builder to construct a valid Multiboot2 information dynamically at runtime.
/// The tags will appear in the order of their corresponding enumeration,
/// except for the END tag.
#[derive(Debug, PartialEq, Eq)]
pub struct InformationBuilder {
    basic_memory_info_tag: Option<BasicMemoryInfoTag>,
    boot_loader_name_tag: Option<BoxedDst<BootLoaderNameTag>>,
    command_line_tag: Option<BoxedDst<CommandLineTag>>,
    efi_boot_services_not_exited_tag: Option<EFIBootServicesNotExitedTag>,
    efi_image_handle32: Option<EFIImageHandle32Tag>,
    efi_image_handle64: Option<EFIImageHandle64Tag>,
    efi_memory_map_tag: Option<BoxedDst<EFIMemoryMapTag>>,
    elf_sections_tag: Option<BoxedDst<ElfSectionsTag>>,
    framebuffer_tag: Option<BoxedDst<FramebufferTag>>,
    image_load_addr: Option<ImageLoadPhysAddrTag>,
    memory_map_tag: Option<BoxedDst<MemoryMapTag>>,
    module_tags: Vec<BoxedDst<ModuleTag>>,
    efisdt32_tag: Option<EFISdt32Tag>,
    efisdt64_tag: Option<EFISdt64Tag>,
    rsdp_v1_tag: Option<RsdpV1Tag>,
    rsdp_v2_tag: Option<RsdpV2Tag>,
    smbios_tags: Vec<BoxedDst<SmbiosTag>>,
}

impl InformationBuilder {
    pub const fn new() -> Self {
        Self {
            basic_memory_info_tag: None,
            boot_loader_name_tag: None,
            command_line_tag: None,
            efisdt32_tag: None,
            efisdt64_tag: None,
            efi_boot_services_not_exited_tag: None,
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
        (size + 7) & !7
    }

    /// Returns the expected length of the boot information, when the
    /// [`Self::build`]-method gets called.
    pub fn expected_len(&self) -> usize {
        let base_len = size_of::<BootInformationHeader>();
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
        if let Some(tag) = &self.efisdt32_tag {
            len += Self::size_or_up_aligned(tag.byte_size())
        }
        if let Some(tag) = &self.efisdt64_tag {
            len += Self::size_or_up_aligned(tag.byte_size())
        }
        if let Some(tag) = &self.efi_boot_services_not_exited_tag {
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
    fn build_add_bytes(dest: &mut Vec<u8>, source: &[u8], is_end_tag: bool) {
        let vec_next_write_ptr = unsafe { dest.as_ptr().add(dest.len()) };
        // At this point, the alignment is guaranteed. If not, something is
        // broken fundamentally.
        assert_eq!(vec_next_write_ptr.align_offset(8), 0);

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
    pub fn build(self) -> BootInformationBytes {
        const ALIGN: usize = 8;

        // PHASE 1/2: Prepare Vector

        // We allocate more than necessary so that we can ensure an correct
        // alignment within this data.
        let expected_len = self.expected_len();
        let alloc_len = expected_len + 7;
        let mut bytes = Vec::<u8>::with_capacity(alloc_len);
        // Pointer to check that no relocation happened.
        let alloc_ptr = bytes.as_ptr();

        // As long as there is no nice way in stable Rust to guarantee the
        // alignment of a vector, I add zero bytes at the beginning and the MBI
        // might not start at the start of the allocation.
        //
        // Unfortunately, it is not possible to reliably test this in a unit
        // test as long as the allocator_api feature is not stable.
        // Due to my manual testing, however, it works.
        let offset = bytes.as_ptr().align_offset(ALIGN);
        bytes.extend([0].repeat(offset));

        // -----------------------------------------------
        // PHASE 2/2: Add Tags
        self.build_add_tags(&mut bytes);

        assert_eq!(
            alloc_ptr,
            bytes.as_ptr(),
            "Vector was reallocated. Alignment of MBI probably broken!"
        );
        assert_eq!(
            bytes[0..offset].iter().sum::<u8>(),
            0,
            "The offset to alignment area should be zero."
        );

        BootInformationBytes {
            offset,
            bytes,
            structure_len: expected_len,
        }
    }

    /// Helper method that adds all the tags to the given vector.
    fn build_add_tags(&self, bytes: &mut Vec<u8>) {
        Self::build_add_bytes(
            bytes,
            // important that we write the correct expected length into the header!
            &BootInformationHeader::new(self.expected_len() as u32).struct_as_bytes(),
            false,
        );
        if let Some(tag) = self.basic_memory_info_tag.as_ref() {
            Self::build_add_bytes(bytes, &tag.struct_as_bytes(), false)
        }
        if let Some(tag) = self.boot_loader_name_tag.as_ref() {
            Self::build_add_bytes(bytes, &tag.struct_as_bytes(), false)
        }
        if let Some(tag) = self.command_line_tag.as_ref() {
            Self::build_add_bytes(bytes, &tag.struct_as_bytes(), false)
        }
        if let Some(tag) = self.efisdt32_tag.as_ref() {
            Self::build_add_bytes(bytes, &tag.struct_as_bytes(), false)
        }
        if let Some(tag) = self.efisdt64_tag.as_ref() {
            Self::build_add_bytes(bytes, &tag.struct_as_bytes(), false)
        }
        if let Some(tag) = self.efi_boot_services_not_exited_tag.as_ref() {
            Self::build_add_bytes(bytes, &tag.struct_as_bytes(), false)
        }
        if let Some(tag) = self.efi_image_handle32.as_ref() {
            Self::build_add_bytes(bytes, &tag.struct_as_bytes(), false)
        }
        if let Some(tag) = self.efi_image_handle64.as_ref() {
            Self::build_add_bytes(bytes, &tag.struct_as_bytes(), false)
        }
        if let Some(tag) = self.efi_memory_map_tag.as_ref() {
            Self::build_add_bytes(bytes, &tag.struct_as_bytes(), false)
        }
        if let Some(tag) = self.elf_sections_tag.as_ref() {
            Self::build_add_bytes(bytes, &tag.struct_as_bytes(), false)
        }
        if let Some(tag) = self.framebuffer_tag.as_ref() {
            Self::build_add_bytes(bytes, &tag.struct_as_bytes(), false)
        }
        if let Some(tag) = self.image_load_addr.as_ref() {
            Self::build_add_bytes(bytes, &tag.struct_as_bytes(), false)
        }
        if let Some(tag) = self.memory_map_tag.as_ref() {
            Self::build_add_bytes(bytes, &tag.struct_as_bytes(), false)
        }
        for tag in &self.module_tags {
            Self::build_add_bytes(bytes, &tag.struct_as_bytes(), false)
        }
        if let Some(tag) = self.rsdp_v1_tag.as_ref() {
            Self::build_add_bytes(bytes, &tag.struct_as_bytes(), false)
        }
        if let Some(tag) = self.rsdp_v2_tag.as_ref() {
            Self::build_add_bytes(bytes, &tag.struct_as_bytes(), false)
        }
        for tag in &self.smbios_tags {
            Self::build_add_bytes(bytes, &tag.struct_as_bytes(), false)
        }
        Self::build_add_bytes(bytes, &EndTag::default().struct_as_bytes(), true);
    }

    /// Adds a 'basic memory information' tag (represented by [`BasicMemoryInfoTag`]) to the builder.
    pub fn basic_memory_info_tag(mut self, basic_memory_info_tag: BasicMemoryInfoTag) -> Self {
        self.basic_memory_info_tag = Some(basic_memory_info_tag);
        self
    }

    /// Adds a 'bootloader name' tag (represented by [`BootLoaderNameTag`]) to the builder.
    pub fn bootloader_name_tag(
        mut self,
        boot_loader_name_tag: BoxedDst<BootLoaderNameTag>,
    ) -> Self {
        self.boot_loader_name_tag = Some(boot_loader_name_tag);
        self
    }

    /// Adds a 'command line' tag (represented by [`CommandLineTag`]) to the builder.
    pub fn command_line_tag(mut self, command_line_tag: BoxedDst<CommandLineTag>) -> Self {
        self.command_line_tag = Some(command_line_tag);
        self
    }

    /// Adds a 'EFI 32-bit system table pointer' tag (represented by [`EFISdt32Tag`]) to the builder.
    pub fn efisdt32_tag(mut self, efisdt32: EFISdt32Tag) -> Self {
        self.efisdt32_tag = Some(efisdt32);
        self
    }

    /// Adds a 'EFI 64-bit system table pointer' tag (represented by [`EFISdt64Tag`]) to the builder.
    pub fn efisdt64_tag(mut self, efisdt64: EFISdt64Tag) -> Self {
        self.efisdt64_tag = Some(efisdt64);
        self
    }

    /// Adds a 'EFI boot services not terminated' tag (represented by [`EFIBootServicesNotExitedTag`]) to the builder.
    pub fn efi_boot_services_not_exited_tag(mut self) -> Self {
        self.efi_boot_services_not_exited_tag = Some(EFIBootServicesNotExitedTag::new());
        self
    }

    /// Adds a 'EFI 32-bit image handle pointer' tag (represented by [`EFIImageHandle32Tag`]) to the builder.
    pub fn efi_image_handle32(mut self, efi_image_handle32: EFIImageHandle32Tag) -> Self {
        self.efi_image_handle32 = Some(efi_image_handle32);
        self
    }

    /// Adds a 'EFI 64-bit image handle pointer' tag (represented by [`EFIImageHandle64Tag`]) to the builder.
    pub fn efi_image_handle64(mut self, efi_image_handle64: EFIImageHandle64Tag) -> Self {
        self.efi_image_handle64 = Some(efi_image_handle64);
        self
    }

    /// Adds a 'EFI Memory map' tag (represented by [`EFIMemoryMapTag`]) to the builder.
    pub fn efi_memory_map_tag(mut self, efi_memory_map_tag: BoxedDst<EFIMemoryMapTag>) -> Self {
        self.efi_memory_map_tag = Some(efi_memory_map_tag);
        self
    }

    /// Adds a 'ELF-Symbols' tag (represented by [`ElfSectionsTag`]) to the builder.
    pub fn elf_sections_tag(mut self, elf_sections_tag: BoxedDst<ElfSectionsTag>) -> Self {
        self.elf_sections_tag = Some(elf_sections_tag);
        self
    }

    /// Adds a 'Framebuffer info' tag (represented by [`FramebufferTag`]) to the builder.
    pub fn framebuffer_tag(mut self, framebuffer_tag: BoxedDst<FramebufferTag>) -> Self {
        self.framebuffer_tag = Some(framebuffer_tag);
        self
    }

    /// Adds a 'Image load base physical address' tag (represented by [`ImageLoadPhysAddrTag`]) to the builder.
    pub fn image_load_addr(mut self, image_load_addr: ImageLoadPhysAddrTag) -> Self {
        self.image_load_addr = Some(image_load_addr);
        self
    }

    /// Adds a (*none EFI*) 'memory map' tag (represented by [`MemoryMapTag`]) to the builder.
    pub fn memory_map_tag(mut self, memory_map_tag: BoxedDst<MemoryMapTag>) -> Self {
        self.memory_map_tag = Some(memory_map_tag);
        self
    }

    /// Adds a 'Modules' tag (represented by [`ModuleTag`]) to the builder.
    /// This tag can occur multiple times in boot information.
    pub fn add_module_tag(mut self, module_tag: BoxedDst<ModuleTag>) -> Self {
        self.module_tags.push(module_tag);
        self
    }

    /// Adds a 'ACPI old RSDP' tag (represented by [`RsdpV1Tag`]) to the builder.
    pub fn rsdp_v1_tag(mut self, rsdp_v1_tag: RsdpV1Tag) -> Self {
        self.rsdp_v1_tag = Some(rsdp_v1_tag);
        self
    }

    /// Adds a 'ACPI new RSDP' tag (represented by [`RsdpV2Tag`]) to the builder.
    pub fn rsdp_v2_tag(mut self, rsdp_v2_tag: RsdpV2Tag) -> Self {
        self.rsdp_v2_tag = Some(rsdp_v2_tag);
        self
    }

    /// Adds a 'SMBIOS tables' tag (represented by [`SmbiosTag`]) to the builder.
    pub fn add_smbios_tag(mut self, smbios_tag: BoxedDst<SmbiosTag>) -> Self {
        self.smbios_tags.push(smbios_tag);
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::builder::information::InformationBuilder;
    use crate::{BasicMemoryInfoTag, BootInformation, CommandLineTag, ModuleTag};

    fn create_builder() -> InformationBuilder {
        let mut builder = InformationBuilder::new();

        // Multiboot2 basic information + end tag
        let mut expected_len = 8 + 8;
        assert_eq!(builder.expected_len(), expected_len);

        // the most simple tag
        builder = builder.basic_memory_info_tag(BasicMemoryInfoTag::new(640, 7 * 1024));
        expected_len += 16;
        assert_eq!(builder.expected_len(), expected_len);
        // a tag that has a dynamic size
        builder = builder.command_line_tag(CommandLineTag::new("test"));
        expected_len += 8 + 5 + 3; // padding
        assert_eq!(builder.expected_len(), expected_len);
        // many modules
        builder = builder.add_module_tag(ModuleTag::new(0, 1234, "module1"));
        expected_len += 16 + 8;
        assert_eq!(builder.expected_len(), expected_len);
        builder = builder.add_module_tag(ModuleTag::new(5678, 6789, "module2"));
        expected_len += 16 + 8;
        assert_eq!(builder.expected_len(), expected_len);

        println!("builder: {:#?}", builder);
        println!("expected_len: {} bytes", builder.expected_len());

        builder
    }

    #[test]
    fn test_size_or_up_aligned() {
        assert_eq!(0, InformationBuilder::size_or_up_aligned(0));
        assert_eq!(8, InformationBuilder::size_or_up_aligned(1));
        assert_eq!(8, InformationBuilder::size_or_up_aligned(8));
        assert_eq!(16, InformationBuilder::size_or_up_aligned(9));
    }

    /// Test of the `build` method in isolation specifically for miri to check
    /// for memory issues.
    #[test]
    fn test_builder_miri() {
        let builder = create_builder();
        let expected_len = builder.expected_len();
        assert_eq!(builder.build().as_bytes().len(), expected_len);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_builder() {
        // Step 1/2: Build MBI
        let mb2i_data = create_builder().build();

        // Step 2/2: Test the built MBI
        let mb2i = unsafe { BootInformation::load(mb2i_data.as_ptr().cast()) }
            .expect("generated information should be readable");

        assert_eq!(mb2i.basic_memory_info_tag().unwrap().memory_lower(), 640);
        assert_eq!(
            mb2i.basic_memory_info_tag().unwrap().memory_upper(),
            7 * 1024
        );
        assert_eq!(mb2i.command_line_tag().unwrap().cmdline().unwrap(), "test");
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

        // Printing the MBI transitively ensures that a lot of stuff works.
        println!("{:#?}", mb2i);
    }
}
