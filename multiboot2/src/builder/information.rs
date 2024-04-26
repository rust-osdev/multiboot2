//! Exports item [`InformationBuilder`].
use crate::builder::{AsBytes, BoxedDst};
use crate::{
    BasicMemoryInfoTag, BootInformationHeader, BootLoaderNameTag, CommandLineTag,
    EFIBootServicesNotExitedTag, EFIImageHandle32Tag, EFIImageHandle64Tag, EFIMemoryMapTag,
    EFISdt32Tag, EFISdt64Tag, ElfSectionsTag, EndTag, FramebufferTag, ImageLoadPhysAddrTag,
    MemoryMapTag, ModuleTag, RsdpV1Tag, RsdpV2Tag, SmbiosTag, TagTrait, TagType,
};
use alloc::vec::Vec;
use core::fmt::{Display, Formatter};
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

type SerializedTag = Vec<u8>;

/// Error that indicates a tag was added multiple times that is not allowed to
/// be there multiple times.
#[derive(Debug)]
#[allow(unused)]
pub struct RedundantTagError(TagType);

impl Display for RedundantTagError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(feature = "unstable")]
impl core::error::Error for RedundantTagError {}

/// Builder to construct a valid Multiboot2 information dynamically at runtime.
/// The tags will appear in the order of their corresponding enumeration,
/// except for the END tag.
#[derive(Debug, PartialEq, Eq)]
pub struct InformationBuilder(Vec<(TagType, SerializedTag)>);

impl Default for InformationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl InformationBuilder {
    /// Creates a new builder.
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    /// Returns the provided number or the next multiple of 8. This is helpful
    /// to ensure that the following tag starts at a 8-byte aligned boundary.
    const fn size_or_up_aligned(size: usize) -> usize {
        (size + 7) & !7
    }

    /// Returns the expected length of the boot information, when the
    /// [`Self::build`]-method is called. This function assumes that the begin
    /// of the boot information is 8-byte aligned and automatically adds padding
    /// between tags to ensure that each tag is 8-byte aligned.
    pub fn expected_len(&self) -> usize {
        let tag_size_iter = self.0.iter().map(|(_, bytes)| bytes.len());

        let payload_tags_size = tag_size_iter.fold(0, |acc, tag_size| {
            // size_or_up_aligned: make sure next tag is 8-byte aligned
            acc + Self::size_or_up_aligned(tag_size)
        });

        size_of::<BootInformationHeader>() + payload_tags_size + size_of::<EndTag>()
    }

    /// Adds the bytes of a tag to the final Multiboot2 information byte vector.
    fn build_add_tag(dest_buf: &mut Vec<u8>, tag_serialized: &[u8], tag_type: TagType) {
        let vec_next_write_ptr = unsafe { dest_buf.as_ptr().add(dest_buf.len()) };

        // At this point, the alignment is guaranteed. If not, something is
        // broken fundamentally.
        assert_eq!(vec_next_write_ptr.align_offset(8), 0);

        dest_buf.extend(tag_serialized);

        if tag_type != TagType::End {
            let size = tag_serialized.len();
            let size_to_8_align = Self::size_or_up_aligned(size);
            let size_to_8_align_diff = size_to_8_align - size;
            // fill zeroes so that next data block is 8-byte aligned
            dest_buf.extend([0].repeat(size_to_8_align_diff));
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
        bytes.extend(BootInformationHeader::new(self.expected_len() as u32).as_bytes());

        for (tag_type, tag_serialized) in self.0 {
            Self::build_add_tag(&mut bytes, tag_serialized.as_slice(), tag_type)
        }
        Self::build_add_tag(&mut bytes, EndTag::default().as_bytes(), TagType::End);

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

    /// Adds a arbitrary tag that implements [`TagTrait`] to the builder. Only
    /// [`TagType::Module`] and [`TagType::Custom`] are allowed to occur
    /// multiple times. For other tags, this function returns an error.
    ///
    /// It is not required to manually add the [`TagType::End`] tag.
    ///
    /// The tags of the boot information will be ordered naturally, i.e., by
    /// their numeric ID.
    pub fn add_tag<T: TagTrait + ?Sized>(mut self, tag: &T) -> Result<Self, RedundantTagError> {
        // not required to do this manually
        if T::ID == TagType::End {
            return Ok(self);
        }

        let is_redundant_tag = self
            .0
            .iter()
            .map(|(typ, _)| *typ)
            .any(|typ| typ == T::ID && !Self::tag_is_allowed_multiple_times(typ));

        if is_redundant_tag {
            log::debug!(
                "Can't add tag of type {:?}. Only Module tags and Custom tags are allowed to appear multiple times.",
                T::ID
            );
            return Err(RedundantTagError(T::ID));
        }
        self.0.push((T::ID, tag.as_bytes().to_vec()));
        self.0.sort_by_key(|(typ, _)| *typ);

        Ok(self)
    }

    /// Adds a 'basic memory information' tag (represented by [`BasicMemoryInfoTag`]) to the builder.
    pub fn basic_memory_info_tag(self, tag: BasicMemoryInfoTag) -> Self {
        self.add_tag(&tag).unwrap()
    }

    /// Adds a 'bootloader name' tag (represented by [`BootLoaderNameTag`]) to the builder.
    pub fn bootloader_name_tag(self, tag: BoxedDst<BootLoaderNameTag>) -> Self {
        self.add_tag(&*tag).unwrap()
    }

    /// Adds a 'command line' tag (represented by [`CommandLineTag`]) to the builder.
    pub fn command_line_tag(self, tag: BoxedDst<CommandLineTag>) -> Self {
        self.add_tag(&*tag).unwrap()
    }

    /// Adds a 'EFI 32-bit system table pointer' tag (represented by [`EFISdt32Tag`]) to the builder.
    pub fn efisdt32_tag(self, tag: EFISdt32Tag) -> Self {
        self.add_tag(&tag).unwrap()
    }

    /// Adds a 'EFI 64-bit system table pointer' tag (represented by [`EFISdt64Tag`]) to the builder.
    pub fn efisdt64_tag(self, tag: EFISdt64Tag) -> Self {
        self.add_tag(&tag).unwrap()
    }

    /// Adds a 'EFI boot services not terminated' tag (represented by [`EFIBootServicesNotExitedTag`]) to the builder.
    pub fn efi_boot_services_not_exited_tag(self) -> Self {
        self.add_tag(&EFIBootServicesNotExitedTag::new()).unwrap()
    }

    /// Adds a 'EFI 32-bit image handle pointer' tag (represented by [`EFIImageHandle32Tag`]) to the builder.
    pub fn efi_image_handle32(self, tag: EFIImageHandle32Tag) -> Self {
        self.add_tag(&tag).unwrap()
    }

    /// Adds a 'EFI 64-bit image handle pointer' tag (represented by [`EFIImageHandle64Tag`]) to the builder.
    pub fn efi_image_handle64(self, tag: EFIImageHandle64Tag) -> Self {
        self.add_tag(&tag).unwrap()
    }

    /// Adds a 'EFI Memory map' tag (represented by [`EFIMemoryMapTag`]) to the builder.
    pub fn efi_memory_map_tag(self, tag: BoxedDst<EFIMemoryMapTag>) -> Self {
        self.add_tag(&*tag).unwrap()
    }

    /// Adds a 'ELF-Symbols' tag (represented by [`ElfSectionsTag`]) to the builder.
    pub fn elf_sections_tag(self, tag: BoxedDst<ElfSectionsTag>) -> Self {
        self.add_tag(&*tag).unwrap()
    }

    /// Adds a 'Framebuffer info' tag (represented by [`FramebufferTag`]) to the builder.
    pub fn framebuffer_tag(self, tag: BoxedDst<FramebufferTag>) -> Self {
        self.add_tag(&*tag).unwrap()
    }

    /// Adds a 'Image load base physical address' tag (represented by [`ImageLoadPhysAddrTag`]) to the builder.
    pub fn image_load_addr(self, tag: ImageLoadPhysAddrTag) -> Self {
        self.add_tag(&tag).unwrap()
    }

    /// Adds a (*none EFI*) 'memory map' tag (represented by [`MemoryMapTag`]) to the builder.
    pub fn memory_map_tag(self, tag: BoxedDst<MemoryMapTag>) -> Self {
        self.add_tag(&*tag).unwrap()
    }

    /// Adds a 'Modules' tag (represented by [`ModuleTag`]) to the builder.
    /// This tag can occur multiple times in boot information.
    pub fn add_module_tag(self, tag: BoxedDst<ModuleTag>) -> Self {
        self.add_tag(&*tag).unwrap()
    }

    /// Adds a 'ACPI old RSDP' tag (represented by [`RsdpV1Tag`]) to the builder.
    pub fn rsdp_v1_tag(self, tag: RsdpV1Tag) -> Self {
        self.add_tag(&tag).unwrap()
    }

    /// Adds a 'ACPI new RSDP' tag (represented by [`RsdpV2Tag`]) to the builder.
    pub fn rsdp_v2_tag(self, tag: RsdpV2Tag) -> Self {
        self.add_tag(&tag).unwrap()
    }

    /// Adds a 'SMBIOS tables' tag (represented by [`SmbiosTag`]) to the builder.
    pub fn smbios_tag(self, tag: BoxedDst<SmbiosTag>) -> Self {
        self.add_tag(&*tag).unwrap()
    }

    fn tag_is_allowed_multiple_times(tag_type: TagType) -> bool {
        matches!(
            tag_type,
            TagType::Module | TagType::Smbios | TagType::Custom(_)
        )
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
