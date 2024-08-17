//! Module for [`BootInformation`].

#[cfg(feature = "builder")]
use crate::builder::AsBytes;
use crate::framebuffer::UnknownFramebufferType;
use crate::tag::{TagHeader, TagIter};
use crate::{
    module, BasicMemoryInfoTag, BootLoaderNameTag, CommandLineTag, EFIBootServicesNotExitedTag,
    EFIImageHandle32Tag, EFIImageHandle64Tag, EFIMemoryMapTag, EFISdt32Tag, EFISdt64Tag,
    ElfSectionIter, ElfSectionsTag, EndTag, FramebufferTag, ImageLoadPhysAddrTag, MemoryMapTag,
    ModuleIter, RsdpV1Tag, RsdpV2Tag, SmbiosTag, TagTrait, VBEInfoTag,
};
use core::fmt;
use core::mem;
use core::ptr;
use derive_more::Display;

/// Error type that describes errors while loading/parsing a multiboot2 information structure
/// from a given address.
#[derive(Display, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MbiLoadError {
    /// The address is invalid. Make sure that the address is 8-byte aligned,
    /// according to the spec.
    #[display("The address is invalid")]
    IllegalAddress,
    /// The total size of the multiboot2 information structure must be not zero
    /// and a multiple of 8.
    #[display("The size of the MBI is unexpected")]
    IllegalTotalSize(u32),
    /// Missing end tag. Each multiboot2 boot information requires to have an
    /// end tag.
    #[display("There is no end tag")]
    NoEndTag,
}

#[cfg(feature = "unstable")]
impl core::error::Error for MbiLoadError {}

/// The basic header of a [`BootInformation`] as sized Rust type.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C, align(8))]
pub struct BootInformationHeader {
    // size is multiple of 8
    total_size: u32,
    _reserved: u32,
    // Followed by the boot information tags.
}

impl BootInformationHeader {
    #[cfg(feature = "builder")]
    pub(crate) const fn new(total_size: u32) -> Self {
        Self {
            total_size,
            _reserved: 0,
        }
    }

    /// Returns the total size of the structure.
    #[must_use]
    pub const fn total_size(&self) -> u32 {
        self.total_size
    }
}

#[cfg(feature = "builder")]
impl AsBytes for BootInformationHeader {}

/// This type holds the whole data of the MBI. This helps to better satisfy miri
/// when it checks for memory issues.
#[derive(ptr_meta::Pointee)]
#[repr(C, align(8))]
struct BootInformationInner {
    header: BootInformationHeader,
    tags: [u8],
}

impl BootInformationInner {
    /// Checks if the MBI has a valid end tag by checking the end of the mbi's
    /// bytes.
    fn has_valid_end_tag(&self) -> bool {
        let self_ptr = ptr::addr_of!(*self);

        let end_tag_ptr = unsafe {
            self_ptr
                .cast::<u8>()
                .add(self.header.total_size as usize)
                .sub(mem::size_of::<EndTag>())
                .cast::<TagHeader>()
        };
        let end_tag = unsafe { &*end_tag_ptr };

        end_tag.typ == EndTag::ID && end_tag.size as usize == mem::size_of::<EndTag>()
    }
}

/// A Multiboot 2 Boot Information (MBI) accessor.
#[repr(transparent)]
pub struct BootInformation<'a>(&'a BootInformationInner);

impl<'a> BootInformation<'a> {
    /// Loads the [`BootInformation`] from a pointer. The pointer must be valid
    /// and aligned to an 8-byte boundary, as defined by the spec.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use multiboot2::{BootInformation, BootInformationHeader};
    ///
    /// fn kernel_entry(mb_magic: u32, mbi_ptr: u32) {
    ///     if mb_magic == multiboot2::MAGIC {
    ///         let boot_info = unsafe { BootInformation::load(mbi_ptr as *const BootInformationHeader).unwrap() };
    ///         let _cmd = boot_info.command_line_tag();
    ///     } else { /* Panic or use multiboot1 flow. */ }
    /// }
    /// ```
    ///
    /// ## Safety
    /// * `ptr` must be valid for reading. Otherwise this function might cause
    ///   invalid machine state or crash your binary (kernel). This can be the
    ///   case in environments with standard environment (segfault), but also in
    ///   boot environments, such as UEFI.
    /// * The memory at `ptr` must not be modified after calling `load` or the
    ///   program may observe unsynchronized mutation.
    pub unsafe fn load(ptr: *const BootInformationHeader) -> Result<Self, MbiLoadError> {
        // null or not aligned
        if ptr.is_null() || ptr.align_offset(8) != 0 {
            return Err(MbiLoadError::IllegalAddress);
        }

        // mbi: reference to basic header
        let mbi = &*ptr;

        // Check if total size is not 0 and a multiple of 8.
        if mbi.total_size == 0 || mbi.total_size & 0b111 != 0 {
            return Err(MbiLoadError::IllegalTotalSize(mbi.total_size));
        }

        let slice_size = mbi.total_size as usize - mem::size_of::<BootInformationHeader>();
        // mbi: reference to full mbi
        let mbi = ptr_meta::from_raw_parts::<BootInformationInner>(ptr.cast(), slice_size);
        let mbi = &*mbi;

        if !mbi.has_valid_end_tag() {
            return Err(MbiLoadError::NoEndTag);
        }

        Ok(Self(mbi))
    }

    /// Get the start address of the boot info.
    #[must_use]
    pub fn start_address(&self) -> usize {
        self.as_ptr() as usize
    }

    /// Get the start address of the boot info as pointer.
    #[must_use]
    pub const fn as_ptr(&self) -> *const () {
        core::ptr::addr_of!(*self.0).cast()
    }

    /// Get the end address of the boot info.
    ///
    /// This is the same as doing:
    ///
    /// ```rust,no_run
    /// # use multiboot2::{BootInformation, BootInformationHeader};
    /// # let ptr = 0xdeadbeef as *const BootInformationHeader;
    /// # let boot_info = unsafe { BootInformation::load(ptr).unwrap() };
    /// let end_addr = boot_info.start_address() + boot_info.total_size();
    /// ```
    #[must_use]
    pub fn end_address(&self) -> usize {
        self.start_address() + self.total_size()
    }

    /// Get the total size of the boot info struct.
    #[must_use]
    pub const fn total_size(&self) -> usize {
        self.0.header.total_size as usize
    }

    // ######################################################
    // ### BEGIN OF TAG GETTERS (in alphabetical order)

    /*fn apm(&self) {
        // also add to debug output
        todo!()
    }*/

    /// Search for the basic memory info tag.
    #[must_use]
    pub fn basic_memory_info_tag(&self) -> Option<&BasicMemoryInfoTag> {
        self.get_tag::<BasicMemoryInfoTag>()
    }

    /// Search for the BootLoader name tag.
    #[must_use]
    pub fn boot_loader_name_tag(&self) -> Option<&BootLoaderNameTag> {
        self.get_tag::<BootLoaderNameTag>()
    }

    /*fn bootdev(&self) {
        // also add to debug output
        todo!()
    }*/

    /// Search for the Command line tag.
    #[must_use]
    pub fn command_line_tag(&self) -> Option<&CommandLineTag> {
        self.get_tag::<CommandLineTag>()
    }

    /// Search for the EFI boot services not exited tag.
    #[must_use]
    pub fn efi_bs_not_exited_tag(&self) -> Option<&EFIBootServicesNotExitedTag> {
        self.get_tag::<EFIBootServicesNotExitedTag>()
    }

    /// Search for the EFI Memory map tag, if the boot services were exited.
    /// Otherwise, if the [`TagType::EfiBs`] tag is present, this returns `None`
    /// as it is strictly recommended to get the memory map from the `uefi`
    /// services.
    ///
    /// [`TagType::EfiBs`]: crate::TagType::EfiBs
    #[must_use]
    pub fn efi_memory_map_tag(&self) -> Option<&EFIMemoryMapTag> {
        // If the EFIBootServicesNotExited is present, then we should not use
        // the memory map, as it could still be in use.
        self.get_tag::<EFIBootServicesNotExitedTag>().map_or_else(
            || self.get_tag::<EFIMemoryMapTag>(), |_tag| {
                            log::debug!("The EFI memory map is present but the UEFI Boot Services Not Existed Tag is present. Returning None.");
                             None
                        })
    }

    /// Search for the EFI 32-bit SDT tag.
    #[must_use]
    pub fn efi_sdt32_tag(&self) -> Option<&EFISdt32Tag> {
        self.get_tag::<EFISdt32Tag>()
    }

    /// Search for the EFI 64-bit SDT tag.
    #[must_use]
    pub fn efi_sdt64_tag(&self) -> Option<&EFISdt64Tag> {
        self.get_tag::<EFISdt64Tag>()
    }

    /// Search for the EFI 32-bit image handle pointer tag.
    #[must_use]
    pub fn efi_ih32_tag(&self) -> Option<&EFIImageHandle32Tag> {
        self.get_tag::<EFIImageHandle32Tag>()
    }

    /// Search for the EFI 64-bit image handle pointer tag.
    #[must_use]
    pub fn efi_ih64_tag(&self) -> Option<&EFIImageHandle64Tag> {
        self.get_tag::<EFIImageHandle64Tag>()
    }

    /// Returns an [`ElfSectionIter`] iterator over the ELF Sections, if the
    /// [`ElfSectionsTag`] is present.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use multiboot2::{BootInformation, BootInformationHeader};
    /// # let ptr = 0xdeadbeef as *const BootInformationHeader;
    /// # let boot_info = unsafe { BootInformation::load(ptr).unwrap() };
    /// if let Some(sections) = boot_info.elf_sections() {
    ///     let mut total = 0;
    ///     for section in sections {
    ///         println!("Section: {:?}", section);
    ///         total += 1;
    ///     }
    /// }
    /// ```
    #[must_use]
    pub fn elf_sections(&self) -> Option<ElfSectionIter> {
        let tag = self.get_tag::<ElfSectionsTag>();
        tag.map(|t| {
            assert!((t.entry_size() * t.shndx()) <= t.size() as u32);
            t.sections_iter()
        })
    }

    /// Search for the VBE framebuffer tag. The result is `Some(Err(e))`, if the
    /// framebuffer type is unknown, while the framebuffer tag is present.
    #[must_use]
    pub fn framebuffer_tag(&self) -> Option<Result<&FramebufferTag, UnknownFramebufferType>> {
        self.get_tag::<FramebufferTag>()
            .map(|tag| match tag.buffer_type() {
                Ok(_) => Ok(tag),
                Err(e) => Err(e),
            })
    }

    /// Search for the Image Load Base Physical Address tag.
    #[must_use]
    pub fn load_base_addr_tag(&self) -> Option<&ImageLoadPhysAddrTag> {
        self.get_tag::<ImageLoadPhysAddrTag>()
    }

    /// Search for the Memory map tag.
    #[must_use]
    pub fn memory_map_tag(&self) -> Option<&MemoryMapTag> {
        self.get_tag::<MemoryMapTag>()
    }

    /// Get an iterator of all module tags.
    #[must_use]
    pub fn module_tags(&self) -> ModuleIter {
        module::module_iter(self.tags())
    }

    /*fn network_tag(&self) {
        // also add to debug output
        todo!()
    }*/

    /// Search for the (ACPI 1.0) RSDP tag.
    #[must_use]
    pub fn rsdp_v1_tag(&self) -> Option<&RsdpV1Tag> {
        self.get_tag::<RsdpV1Tag>()
    }

    /// Search for the (ACPI 2.0 or later) RSDP tag.
    #[must_use]
    pub fn rsdp_v2_tag(&self) -> Option<&RsdpV2Tag> {
        self.get_tag::<RsdpV2Tag>()
    }

    /// Search for the SMBIOS tag.
    #[must_use]
    pub fn smbios_tag(&self) -> Option<&SmbiosTag> {
        self.get_tag::<SmbiosTag>()
    }

    /// Search for the VBE information tag.
    #[must_use]
    pub fn vbe_info_tag(&self) -> Option<&VBEInfoTag> {
        self.get_tag::<VBEInfoTag>()
    }

    // ### END OF TAG GETTERS
    // ######################################################

    /// Public getter to find any Multiboot tag by its type, including
    /// specified and custom ones.
    ///
    /// # Specified or Custom Tags
    /// The Multiboot2 specification specifies a list of tags, see [`TagType`].
    /// However, it doesn't forbid to use custom tags. Because of this, there
    /// exists the [`TagType`] abstraction. It is recommended to use this
    /// getter only for custom tags. For specified tags, use getters, such as
    /// [`Self::efi_ih64_tag`].
    ///
    /// ## Use Custom Tags
    /// The following example shows how you may use this interface to parse
    /// custom tags from the MBI. If they are dynamically sized (DST), a few more
    /// special handling is required. This is reflected by code-comments.
    ///
    /// ```no_run
    /// use multiboot2::{BootInformation, BootInformationHeader, parse_slice_as_string, StringError, TagHeader, TagTrait, TagType, TagTypeId};
    ///
    /// #[repr(C)]
    /// #[derive(multiboot2::Pointee)] // Only needed for DSTs.
    /// struct CustomTag {
    ///     tag: TagTypeId,
    ///     size: u32,
    ///     // begin of inline string
    ///     name: [u8],
    /// }
    ///
    /// // This implementation is only necessary for tags that are DSTs.
    /// impl TagTrait for CustomTag {
    ///     const ID: TagType = TagType::Custom(0x1337);
    ///
    ///     fn dst_len(header: &TagHeader) -> usize {
    ///         // The size of the sized portion of the custom tag.
    ///         let tag_base_size = 8; // id + size is 8 byte in size
    ///         assert!(header.size >= 8);
    ///         header.size as usize - tag_base_size
    ///     }
    /// }
    ///
    /// impl CustomTag {
    ///     fn name(&self) -> Result<&str, StringError> {
    ///         parse_slice_as_string(&self.name)
    ///     }
    /// }
    /// let mbi_ptr = 0xdeadbeef as *const BootInformationHeader;
    /// let mbi = unsafe { BootInformation::load(mbi_ptr).unwrap() };
    ///
    /// let tag = mbi
    ///     .get_tag::<CustomTag>()
    ///     .unwrap();
    /// assert_eq!(tag.name(), Ok("name"));
    /// ```
    ///
    /// [`TagType`]: crate::TagType
    #[must_use]
    pub fn get_tag<TagT: TagTrait + ?Sized + 'a>(&'a self) -> Option<&'a TagT> {
        self.tags()
            .find(|tag| tag.header().typ == TagT::ID)
            .map(|tag| tag.cast::<TagT>())
    }

    /// Returns an iterator over all tags.
    fn tags(&self) -> TagIter {
        TagIter::new(&self.0.tags)
    }
}

impl fmt::Debug for BootInformation<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        /// Limit how many Elf-Sections should be debug-formatted.
        /// Can be thousands of sections for a Rust binary => this is useless output.
        /// If the user really wants this, they should debug-format the field directly.
        const ELF_SECTIONS_LIMIT: usize = 7;

        let mut debug = f.debug_struct("Multiboot2BootInformation");
        debug
            .field("start_address", &self.start_address())
            .field("end_address", &self.end_address())
            .field("total_size", &self.total_size())
            // now tags in alphabetical order
            .field("basic_memory_info", &(self.basic_memory_info_tag()))
            .field("boot_loader_name", &self.boot_loader_name_tag())
            // .field("bootdev", &self.bootdev_tag())
            .field("command_line", &self.command_line_tag())
            .field("efi_bs_not_exited", &self.efi_bs_not_exited_tag())
            .field("efi_memory_map", &self.efi_memory_map_tag())
            .field("efi_sdt32", &self.efi_sdt32_tag())
            .field("efi_sdt64", &self.efi_sdt64_tag())
            .field("efi_ih32", &self.efi_ih32_tag())
            .field("efi_ih64", &self.efi_ih64_tag());

        // usually this is REALLY big (thousands of tags) => skip it here
        {
            let elf_sections_tag_entries_count =
                self.elf_sections().map(|x| x.count()).unwrap_or(0);

            if elf_sections_tag_entries_count > ELF_SECTIONS_LIMIT {
                debug.field("elf_sections (count)", &elf_sections_tag_entries_count);
            } else {
                debug.field("elf_sections", &self.elf_sections());
            }
        }

        debug
            .field("framebuffer", &self.framebuffer_tag())
            .field("load_base_addr", &self.load_base_addr_tag())
            .field("memory_map", &self.memory_map_tag())
            .field("modules", &self.module_tags())
            // .field("network", &self.network_tag())
            .field("rsdp_v1", &self.rsdp_v1_tag())
            .field("rsdp_v2", &self.rsdp_v2_tag())
            .field("smbios_tag", &self.smbios_tag())
            .field("vbe_info_tag", &self.vbe_info_tag())
            .finish()
    }
}
