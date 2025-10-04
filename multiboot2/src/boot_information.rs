//! Module for [`BootInformation`].

use crate::framebuffer::UnknownFramebufferType;
use crate::tag::TagHeader;
use crate::{
    ApmTag, BasicMemoryInfoTag, BootLoaderNameTag, BootdevTag, CommandLineTag,
    EFIBootServicesNotExitedTag, EFIImageHandle32Tag, EFIImageHandle64Tag, EFIMemoryMapTag,
    EFISdt32Tag, EFISdt64Tag, ElfSectionIter, ElfSectionsTag, EndTag, FramebufferTag,
    ImageLoadPhysAddrTag, MemoryMapTag, ModuleIter, NetworkTag, RsdpV1Tag, RsdpV2Tag, SmbiosTag,
    TagIter, TagType, VBEInfoTag, module,
};
use core::fmt;
use core::mem;
use core::ptr::NonNull;
use multiboot2_common::{DynSizedStructure, Header, MaybeDynSized, MemoryError, Tag};
use thiserror::Error;

/// Errors that occur when a chunk of memory can't be parsed as
/// [`BootInformation`].
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Error)]
pub enum LoadError {
    /// The provided memory can't be parsed as [`BootInformation`].
    /// See [`MemoryError`].
    #[error("memory can't be parsed as boot information")]
    Memory(#[source] MemoryError),
    /// Missing mandatory end tag.
    #[error("missing mandatory end tag")]
    NoEndTag,
}

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

impl Header for BootInformationHeader {
    fn payload_len(&self) -> usize {
        self.total_size as usize - mem::size_of::<Self>()
    }

    fn set_size(&mut self, total_size: usize) {
        self.total_size = total_size as u32;
    }
}

/// A Multiboot 2 Boot Information (MBI) accessor.
#[repr(transparent)]
pub struct BootInformation<'a>(&'a DynSizedStructure<BootInformationHeader>);

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
    /// * `ptr` must be valid for reading. Otherwise, this function might cause
    ///   invalid machine state or crash your binary (kernel). This can be the
    ///   case in environments with standard environment (segfault), but also in
    ///   boot environments, such as UEFI.
    /// * The memory at `ptr` must not be modified after calling `load` or the
    ///   program may observe unsynchronized mutation.
    pub unsafe fn load(ptr: *const BootInformationHeader) -> Result<Self, LoadError> {
        let ptr = NonNull::new(ptr.cast_mut()).ok_or(LoadError::Memory(MemoryError::Null))?;
        let inner = unsafe { DynSizedStructure::ref_from_ptr(ptr).map_err(LoadError::Memory)? };

        let this = Self(inner);
        if !this.has_valid_end_tag() {
            return Err(LoadError::NoEndTag);
        }
        Ok(this)
    }

    /// Checks if the MBI has a valid end tag by checking the end of the mbi's
    /// bytes.
    fn has_valid_end_tag(&self) -> bool {
        let header = self.0.header();
        let end_tag_ptr = unsafe {
            self.0
                .payload()
                .as_ptr()
                .add(header.payload_len())
                .sub(mem::size_of::<EndTag>())
                .cast::<TagHeader>()
        };
        let end_tag = unsafe { &*end_tag_ptr };

        end_tag.typ == EndTag::ID && end_tag.size as usize == mem::size_of::<EndTag>()
    }

    /// Get the start address of the boot info.
    #[must_use]
    // TODO deprecated and use pointers only (see provenance discussions)
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
    // TODO deprecated and use pointers only (see provenance discussions)
    pub fn end_address(&self) -> usize {
        self.start_address() + self.total_size()
    }

    /// Get the total size of the boot info struct.
    #[must_use]
    pub const fn total_size(&self) -> usize {
        self.0.header().total_size as usize
    }

    // ######################################################
    // ### BEGIN OF TAG GETTERS (in alphabetical order)

    /// Search for the [`ApmTag`].
    #[must_use]
    pub fn apm_tag(&self) -> Option<&ApmTag> {
        self.get_tag::<ApmTag>()
    }

    /// Search for the [`BasicMemoryInfoTag`].
    #[must_use]
    pub fn basic_memory_info_tag(&self) -> Option<&BasicMemoryInfoTag> {
        self.get_tag::<BasicMemoryInfoTag>()
    }

    /// Search for the [`BootLoaderNameTag`].
    #[must_use]
    pub fn boot_loader_name_tag(&self) -> Option<&BootLoaderNameTag> {
        self.get_tag::<BootLoaderNameTag>()
    }

    /// Search for the [`BootdevTag`].
    #[must_use]
    pub fn bootdev_tag(&self) -> Option<&BootdevTag> {
        self.get_tag::<BootdevTag>()
    }

    /// Search for the [`CommandLineTag`].
    #[must_use]
    pub fn command_line_tag(&self) -> Option<&CommandLineTag> {
        self.get_tag::<CommandLineTag>()
    }

    /// Search for the [`EFIBootServicesNotExitedTag`].
    #[must_use]
    pub fn efi_bs_not_exited_tag(&self) -> Option<&EFIBootServicesNotExitedTag> {
        self.get_tag::<EFIBootServicesNotExitedTag>()
    }

    /// Search for the [`EFIMemoryMapTag`], if the boot services were exited.
    /// Otherwise, if the [`TagType::EfiBs`] tag is present, this returns `None`
    /// as it is strictly recommended to get the memory map from `uefi`
    /// instead.
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

    /// Search for the [`EFISdt32Tag`].
    #[must_use]
    pub fn efi_sdt32_tag(&self) -> Option<&EFISdt32Tag> {
        self.get_tag::<EFISdt32Tag>()
    }

    /// Search for the [`EFISdt64Tag`].
    #[must_use]
    pub fn efi_sdt64_tag(&self) -> Option<&EFISdt64Tag> {
        self.get_tag::<EFISdt64Tag>()
    }

    /// Search for the [`EFIImageHandle32Tag`].
    #[must_use]
    pub fn efi_ih32_tag(&self) -> Option<&EFIImageHandle32Tag> {
        self.get_tag::<EFIImageHandle32Tag>()
    }

    /// Search for the [`EFIImageHandle64Tag`].
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
    /// if let Some(sections) = boot_info.elf_sections_tag().map(|tag| tag.sections()) {
    ///     let mut total = 0;
    ///     for section in sections {
    ///         println!("Section: {:?}", section);
    ///         total += 1;
    ///     }
    /// }
    /// ```
    #[must_use]
    #[deprecated = "Use elf_sections_tag() instead and corresponding getters"]
    pub fn elf_sections(&self) -> Option<ElfSectionIter<'_>> {
        let tag = self.get_tag::<ElfSectionsTag>();
        tag.map(|t| {
            assert!((t.entry_size() * t.shndx()) <= t.header().size);
            t.sections()
        })
    }

    /// Search for the [`ElfSectionsTag`].
    #[must_use]
    pub fn elf_sections_tag(&self) -> Option<&ElfSectionsTag> {
        self.get_tag()
    }

    /// Search for the [`FramebufferTag`]. The result is `Some(Err(e))`, if the
    /// framebuffer type is unknown, while the framebuffer tag is present.
    #[must_use]
    pub fn framebuffer_tag(&self) -> Option<Result<&FramebufferTag, UnknownFramebufferType>> {
        self.get_tag::<FramebufferTag>()
            .map(|tag| match tag.buffer_type() {
                Ok(_) => Ok(tag),
                Err(e) => Err(e),
            })
    }

    /// Search for the [`ImageLoadPhysAddrTag`].
    #[must_use]
    pub fn load_base_addr_tag(&self) -> Option<&ImageLoadPhysAddrTag> {
        self.get_tag::<ImageLoadPhysAddrTag>()
    }

    /// Search for the [`MemoryMapTag`].
    #[must_use]
    pub fn memory_map_tag(&self) -> Option<&MemoryMapTag> {
        self.get_tag::<MemoryMapTag>()
    }

    /// Get an iterator of all [`ModuleTag`]s.
    ///
    /// [`ModuleTag`]: crate::ModuleTag
    #[must_use]
    pub fn module_tags(&self) -> ModuleIter<'_> {
        module::module_iter(self.tags())
    }

    /// Search for the [`NetworkTag`].
    #[must_use]
    pub fn network_tag(&self) -> Option<&NetworkTag> {
        self.get_tag::<NetworkTag>()
    }

    /// Search for the [`RsdpV1Tag`].
    #[must_use]
    pub fn rsdp_v1_tag(&self) -> Option<&RsdpV1Tag> {
        self.get_tag::<RsdpV1Tag>()
    }

    /// Search for the [`RsdpV2Tag`].
    #[must_use]
    pub fn rsdp_v2_tag(&self) -> Option<&RsdpV2Tag> {
        self.get_tag::<RsdpV2Tag>()
    }

    /// Search for the [`SmbiosTag`].
    #[must_use]
    pub fn smbios_tag(&self) -> Option<&SmbiosTag> {
        self.get_tag::<SmbiosTag>()
    }

    /// Search for the [`VBEInfoTag`].
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
    /// use std::mem;
    /// use multiboot2::{BootInformation, BootInformationHeader, parse_slice_as_string, StringError, TagHeader, TagType, TagTypeId};    ///
    /// use multiboot2_common::{MaybeDynSized, Tag};
    ///
    /// #[repr(C)]
    /// #[derive(multiboot2::Pointee)] // Only needed for DSTs.
    /// struct CustomTag {
    ///     header: TagHeader,
    ///     some_other_prop: u32,
    ///     // Begin of C string, for example.
    ///     name: [u8],
    /// }
    ///
    /// impl CustomTag {
    ///     fn name(&self) -> Result<&str, StringError> {
    ///         parse_slice_as_string(&self.name)
    ///     }
    /// }
    ///
    /// // Give the library hints how big this tag is.
    /// impl MaybeDynSized for CustomTag {
    ///     type Header = TagHeader;
    ///     const BASE_SIZE: usize = mem::size_of::<TagHeader>() + mem::size_of::<u32>();
    ///
    ///     // This differs for DSTs and normal structs. See function
    ///     // documentation.
    ///     fn dst_len(header: &TagHeader) -> usize {
    ///         assert!(header.size >= Self::BASE_SIZE as u32);
    ///         header.size as usize - Self::BASE_SIZE
    ///     }
    /// }
    ///
    /// // Make the Tag identifiable.
    /// impl Tag for CustomTag {
    ///     type IDType = TagType;
    ///     const ID: TagType = TagType::Custom(0x1337);
    /// }
    ///
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
    pub fn get_tag<T: Tag<IDType = TagType, Header = TagHeader> + ?Sized + 'a>(
        &'a self,
    ) -> Option<&'a T> {
        self.tags()
            .find(|tag| tag.header().typ == T::ID)
            .map(|tag| tag.cast::<T>())
    }

    /// Returns an iterator over all tags.
    ///
    /// This is public to enable users to iterate over tags that appear multiple
    /// times, even tho this is unusual. However, it is recommended to use the
    /// tag getters as normal bootloaders provide most tags only once.
    #[must_use]
    pub fn tags(&self) -> TagIter<'_> {
        TagIter::new(self.0.payload())
    }
}

impl fmt::Debug for BootInformation<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut debug = f.debug_struct("Multiboot2BootInformation");
        debug
            .field("start_address", &self.start_address())
            .field("end_address", &self.end_address())
            .field("total_size", &self.total_size())
            // now tags in alphabetical order
            .field("apm", &self.apm_tag())
            .field("basic_memory_info", &(self.basic_memory_info_tag()))
            .field("boot_loader_name", &self.boot_loader_name_tag())
            .field("bootdev", &self.bootdev_tag())
            .field("command_line", &self.command_line_tag())
            .field("efi_bs_not_exited", &self.efi_bs_not_exited_tag())
            .field("efi_ih32", &self.efi_ih32_tag())
            .field("efi_ih64", &self.efi_ih64_tag())
            .field("efi_memory_map", &self.efi_memory_map_tag())
            .field("efi_sdt32", &self.efi_sdt32_tag())
            .field("efi_sdt64", &self.efi_sdt64_tag())
            .field("elf_sections", &self.elf_sections_tag())
            .field("framebuffer", &self.framebuffer_tag())
            .field("load_base_addr", &self.load_base_addr_tag())
            .field("memory_map", &self.memory_map_tag())
            .field("modules", &self.module_tags())
            .field("network", &self.network_tag())
            .field("rsdp_v1", &self.rsdp_v1_tag())
            .field("rsdp_v2", &self.rsdp_v2_tag())
            .field("smbios", &self.smbios_tag())
            .field("vbe_info", &self.vbe_info_tag())
            // computed fields
            .field("custom_tags_count", &{
                self.tags()
                    .filter(|tag| {
                        let id: TagType = tag.header().typ.into();
                        matches!(id, TagType::Custom(_))
                    })
                    .count()
            })
            .finish()
    }
}
