#![no_std]
#![cfg_attr(feature = "unstable", feature(error_in_core))]
#![deny(missing_debug_implementations)]
// --- BEGIN STYLE CHECKS ---
// These checks are optional in CI for PRs, as discussed in
// https://github.com/rust-osdev/multiboot2/pull/92
#![deny(clippy::all)]
#![deny(rustdoc::all)]
#![allow(rustdoc::private_doc_tests)]
// --- END STYLE CHECKS ---

//! Library that assists parsing the Multiboot2 Information Structure (MBI) from
//! Multiboot2-compliant bootloaders, such as GRUB. It supports all tags from the
//! specification including full support for the sections of ELF files. This library
//! is `no_std` and can be used in a Multiboot2-kernel.
//!
//! The GNU Multiboot(2) specification aims to provide a standardised
//! method of sharing commonly used information about the host machine at
//! boot time and give the payload, i.e. a kernel, a well defined machine
//! state.
//!
//! ## Example
//!
//! ```rust
//! use multiboot2::{BootInformation, BootInformationHeader};
//!
//! fn kernel_entry(mb_magic: u32, mbi_ptr: u32) {
//!     if mb_magic == multiboot2::MAGIC {
//!         let boot_info = unsafe { BootInformation::load(mbi_ptr as *const BootInformationHeader).unwrap() };
//!         let _cmd = boot_info.command_line_tag();
//!     } else { /* Panic or use multiboot1 flow. */ }
//! }
//! ```
//!
//! ## MSRV
//! The MSRV is 1.70.0 stable.

#[cfg(feature = "builder")]
extern crate alloc;

// this crate can use std in tests only
#[cfg_attr(test, macro_use)]
#[cfg(test)]
extern crate std;

#[macro_use]
extern crate bitflags;

#[cfg(feature = "builder")]
pub mod builder;

mod boot_loader_name;
mod command_line;
mod efi;
mod elf_sections;
mod end;
mod framebuffer;
mod image_load_addr;
mod memory_map;
mod module;
mod rsdp;
mod smbios;
mod tag;
mod tag_trait;
mod tag_type;
mod vbe_info;

pub use boot_loader_name::BootLoaderNameTag;
pub use command_line::CommandLineTag;
pub use efi::{
    EFIBootServicesNotExitedTag, EFIImageHandle32Tag, EFIImageHandle64Tag, EFISdt32Tag, EFISdt64Tag,
};
pub use elf_sections::{
    ElfSection, ElfSectionFlags, ElfSectionIter, ElfSectionType, ElfSectionsTag,
};
pub use end::EndTag;
pub use framebuffer::{FramebufferColor, FramebufferField, FramebufferTag, FramebufferType};
pub use image_load_addr::ImageLoadPhysAddrTag;
pub use memory_map::{
    BasicMemoryInfoTag, EFIMemoryAreaType, EFIMemoryDesc, EFIMemoryMapTag, MemoryArea,
    MemoryAreaType, MemoryAreaTypeId, MemoryMapTag,
};
pub use module::{ModuleIter, ModuleTag};
pub use ptr_meta::Pointee;
pub use rsdp::{RsdpV1Tag, RsdpV2Tag};
pub use smbios::SmbiosTag;
pub use tag::{StringError, Tag};
pub use tag_trait::TagTrait;
pub use tag_type::{TagType, TagTypeId};
pub use vbe_info::{
    VBECapabilities, VBEControlInfo, VBEDirectColorAttributes, VBEField, VBEInfoTag,
    VBEMemoryModel, VBEModeAttributes, VBEModeInfo, VBEWindowAttributes,
};

use core::fmt;
use core::mem::size_of;
use derive_more::Display;
// Must be public so that custom tags can be DSTs.
#[cfg(feature = "builder")]
use crate::builder::AsBytes;
use crate::framebuffer::UnknownFramebufferType;
use tag::{TagIter, TagIterMut};

/// Magic number that a Multiboot2-compliant boot loader will use to identify
/// the handoff. The location depends on the architecture and the targeted
/// machine state.
pub const MAGIC: u32 = 0x36d76289;

/// Error type that describes errors while loading/parsing a multiboot2 information structure
/// from a given address.
#[derive(Display, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MbiLoadError {
    /// The address is invalid. Make sure that the address is 8-byte aligned,
    /// according to the spec.
    #[display(fmt = "The address is invalid")]
    IllegalAddress,
    /// The total size of the multiboot2 information structure must be not zero
    /// and a multiple of 8.
    #[display(fmt = "The size of the MBI is unexpected")]
    IllegalTotalSize(u32),
    /// Missing end tag. Each multiboot2 boot information requires to have an
    /// end tag.
    #[display(fmt = "There is no end tag")]
    NoEndTag,
}

#[cfg(feature = "unstable")]
impl core::error::Error for MbiLoadError {}

/// The basic header of a boot information.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct BootInformationHeader {
    // size is multiple of 8
    pub total_size: u32,
    _reserved: u32,
    // Followed by the boot information tags.
}

#[cfg(feature = "builder")]
impl BootInformationHeader {
    fn new(total_size: u32) -> Self {
        Self {
            total_size,
            _reserved: 0,
        }
    }
}

#[cfg(feature = "builder")]
impl AsBytes for BootInformationHeader {}

/// This type holds the whole data of the MBI. This helps to better satisfy miri
/// when it checks for memory issues.
#[derive(ptr_meta::Pointee, Debug)]
#[repr(C)]
pub struct BootInformationInner {
    header: BootInformationHeader,
    tags: [u8],
}

impl BootInformationInner {
    /// Checks if the MBI has a valid end tag by checking the end of the mbi's
    /// bytes.
    fn has_valid_end_tag(&self) -> bool {
        let end_tag_prototype = EndTag::default();

        let self_ptr = unsafe { self.tags.as_ptr().sub(size_of::<BootInformationHeader>()) };

        let end_tag_ptr = unsafe {
            self_ptr
                .add(self.header.total_size as usize)
                .sub(size_of::<EndTag>())
        };
        let end_tag = unsafe { &*(end_tag_ptr as *const EndTag) };

        end_tag.typ == end_tag_prototype.typ && end_tag.size == end_tag_prototype.size
    }
}

impl AsRef<BootInformationInner> for BootInformationInner {
    fn as_ref(&self) -> &BootInformationInner {
        self
    }
}

impl AsMut<BootInformationInner> for BootInformationInner {
    fn as_mut(&mut self) -> &mut BootInformationInner {
        self
    }
}
/// A Multiboot 2 Boot Information (MBI) accessor.
#[repr(transparent)]
pub struct BootInformation<T: AsRef<BootInformationInner>>(T);

impl BootInformation<&BootInformationInner> {
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

        let slice_size = mbi.total_size as usize - size_of::<BootInformationHeader>();
        // mbi: reference to full mbi
        let mbi = ptr_meta::from_raw_parts::<BootInformationInner>(ptr.cast(), slice_size);
        let mbi = &*mbi;

        if !mbi.has_valid_end_tag() {
            return Err(MbiLoadError::NoEndTag);
        }

        Ok(Self(mbi))
    }
}

impl BootInformation<&mut BootInformationInner> {
    /// `BootInformation::load`, but mutably.
    ///
    /// # Safety
    /// The same considerations that apply to `load` also apply here, but the
    /// memory can be modified (through the `_mut` methods).
    pub unsafe fn load_mut(ptr: *mut BootInformationHeader) -> Result<Self, MbiLoadError> {
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

        let slice_size = mbi.total_size as usize - size_of::<BootInformationHeader>();
        // mbi: reference to full mbi
        let mbi = ptr_meta::from_raw_parts_mut::<BootInformationInner>(ptr.cast(), slice_size);
        let mbi = &mut *mbi;

        if !mbi.has_valid_end_tag() {
            return Err(MbiLoadError::NoEndTag);
        }

        Ok(Self(mbi))
    }
}

impl<T: AsRef<BootInformationInner>> BootInformation<T> {
    /// Get the start address of the boot info.
    pub fn start_address(&self) -> usize {
        self.as_ptr() as usize
    }

    /// Get the start address of the boot info as pointer.
    pub fn as_ptr(&self) -> *const () {
        core::ptr::addr_of!(*self.0.as_ref()).cast()
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
    pub fn end_address(&self) -> usize {
        self.start_address() + self.total_size()
    }

    /// Get the total size of the boot info struct.
    pub fn total_size(&self) -> usize {
        self.0.as_ref().header.total_size as usize
    }

    // ######################################################
    // ### BEGIN OF TAG GETTERS (in alphabetical order)

    /*fn apm(&self) {
        // also add to debug output
        todo!()
    }*/

    /// Search for the basic memory info tag.
    pub fn basic_memory_info_tag(&self) -> Option<&BasicMemoryInfoTag> {
        self.get_tag::<BasicMemoryInfoTag>()
    }

    /// Search for the BootLoader name tag.
    pub fn boot_loader_name_tag(&self) -> Option<&BootLoaderNameTag> {
        self.get_tag::<BootLoaderNameTag>()
    }

    /*fn bootdev(&self) {
        // also add to debug output
        todo!()
    }*/

    /// Search for the Command line tag.
    pub fn command_line_tag(&self) -> Option<&CommandLineTag> {
        self.get_tag::<CommandLineTag>()
    }

    /// Search for the EFI boot services not exited tag.
    pub fn efi_bs_not_exited_tag(&self) -> Option<&EFIBootServicesNotExitedTag> {
        self.get_tag::<EFIBootServicesNotExitedTag>()
    }

    /// Search for the EFI Memory map tag, if the boot services were exited.
    /// Otherwise, if the [`TagType::EfiBs`] tag is present, this returns `None`
    /// as it is strictly recommended to get the memory map from the `uefi`
    /// services.
    pub fn efi_memory_map_tag(&self) -> Option<&EFIMemoryMapTag> {
        // If the EFIBootServicesNotExited is present, then we should not use
        // the memory map, as it could still be in use.
        match self.get_tag::<EFIBootServicesNotExitedTag>() {
            Some(_tag) => None,
            None => self.get_tag::<EFIMemoryMapTag>(),
        }
    }

    /// Search for the EFI 32-bit SDT tag.
    pub fn efi_sdt32_tag(&self) -> Option<&EFISdt32Tag> {
        self.get_tag::<EFISdt32Tag>()
    }

    /// Search for the EFI 64-bit SDT tag.
    pub fn efi_sdt64_tag(&self) -> Option<&EFISdt64Tag> {
        self.get_tag::<EFISdt64Tag>()
    }

    /// Search for the EFI 32-bit image handle pointer tag.
    pub fn efi_ih32_tag(&self) -> Option<&EFIImageHandle32Tag> {
        self.get_tag::<EFIImageHandle32Tag>()
    }

    /// Search for the EFI 64-bit image handle pointer tag.
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
    pub fn elf_sections(&self) -> Option<ElfSectionIter> {
        let tag = self.get_tag::<ElfSectionsTag>();
        tag.map(|t| {
            assert!((t.entry_size * t.shndx) <= t.size);
            t.sections()
        })
    }

    /// Search for the VBE framebuffer tag. The result is `Some(Err(e))`, if the
    /// framebuffer type is unknown, while the framebuffer tag is present.
    pub fn framebuffer_tag(&self) -> Option<Result<&FramebufferTag, UnknownFramebufferType>> {
        self.get_tag::<FramebufferTag>()
            .map(|tag| match tag.buffer_type() {
                Ok(_) => Ok(tag),
                Err(e) => Err(e),
            })
    }

    /// Search for the Image Load Base Physical Address tag.
    pub fn load_base_addr_tag(&self) -> Option<&ImageLoadPhysAddrTag> {
        self.get_tag::<ImageLoadPhysAddrTag>()
    }

    /// Search for the Memory map tag.
    pub fn memory_map_tag(&self) -> Option<&MemoryMapTag> {
        self.get_tag::<MemoryMapTag>()
    }

    /// Get an iterator of all module tags.
    pub fn module_tags(&self) -> ModuleIter {
        module::module_iter(self.tags())
    }

    /*fn network_tag(&self) {
        // also add to debug output
        todo!()
    }*/

    /// Search for the (ACPI 1.0) RSDP tag.
    pub fn rsdp_v1_tag(&self) -> Option<&RsdpV1Tag> {
        self.get_tag::<RsdpV1Tag>()
    }

    /// Search for the (ACPI 2.0 or later) RSDP tag.
    pub fn rsdp_v2_tag(&self) -> Option<&RsdpV2Tag> {
        self.get_tag::<RsdpV2Tag>()
    }

    /// Search for the SMBIOS tag.
    pub fn smbios_tag(&self) -> Option<&SmbiosTag> {
        self.get_tag::<SmbiosTag>()
    }

    /// Search for the VBE information tag.
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
    /// use std::str::Utf8Error;
    /// use multiboot2::{BootInformation, BootInformationHeader, Tag, TagTrait, TagType, TagTypeId};
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
    ///     fn dst_size(base_tag: &Tag) -> usize {
    ///         // The size of the sized portion of the custom tag.
    ///         let tag_base_size = 8; // id + size is 8 byte in size
    ///         assert!(base_tag.size >= 8);
    ///         base_tag.size as usize - tag_base_size
    ///     }
    /// }
    ///
    /// impl CustomTag {
    ///     fn name(&self) -> Result<&str, Utf8Error> {
    ///         Tag::parse_slice_as_string(&self.name)
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
    pub fn get_tag<TagT: TagTrait + ?Sized>(&self) -> Option<&TagT> {
        self.tags()
            .find(|tag| tag.typ == TagT::ID)
            .map(|tag| tag.cast_tag::<TagT>())
    }

    /// Returns an iterator over all tags.
    fn tags(&self) -> TagIter {
        TagIter::new(&self.0.as_ref().tags)
    }
}

impl<T: AsRef<BootInformationInner> + AsMut<BootInformationInner>> BootInformation<T> {
    /// Search for the Memory map tag, return a mutable reference.
    pub fn memory_map_tag_mut(&mut self) -> Option<&mut MemoryMapTag> {
        self.get_tag_mut::<MemoryMapTag, _>(TagType::Mmap)
    }

    /// Search for the basic memory info tag, return a mutable reference.
    pub fn basic_memory_info_tag_mut(&mut self) -> Option<&mut BasicMemoryInfoTag> {
        self.get_tag_mut::<BasicMemoryInfoTag, _>(TagType::BasicMeminfo)
    }

    /// Search for the EFI Memory map tag, return a mutable reference.
    pub fn efi_memory_map_tag_mut(&mut self) -> Option<&mut EFIMemoryMapTag> {
        self.get_tag_mut::<EFIMemoryMapTag, _>(TagType::EfiMmap)
    }

    fn get_tag_mut<TagT: TagTrait + ?Sized, TagType: Into<TagTypeId>>(
        &mut self,
        typ: TagType,
    ) -> Option<&mut TagT> {
        let typ = typ.into();
        self.tags_mut()
            .find(|tag| tag.typ == typ)
            .map(|tag| tag.cast_tag_mut::<TagT>())
    }

    fn tags_mut(&mut self) -> TagIterMut {
        TagIterMut::new(&mut self.0.as_mut().tags)
    }
}

// SAFETY: BootInformation contains a const ptr to memory that is never mutated.
// Sending this pointer to other threads is sound.
unsafe impl<T: AsRef<BootInformationInner>> Send for BootInformation<T> {}

impl<T: AsRef<BootInformationInner>> fmt::Debug for BootInformation<T> {
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
                debug.field("elf_sections", &self.elf_sections().unwrap_or_default());
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory_map::MemoryAreaType;
    use crate::tag::StringError;

    /// Compile time test to check if the boot information is Send and Sync.
    /// This test is relevant to give library users flexebility in passing the
    /// struct around.
    #[test]
    fn boot_information_is_send_and_sync() {
        fn accept<T: Send + Sync>(_: T) {}

        #[repr(C, align(8))]
        struct Bytes([u8; 16]);
        let bytes: Bytes = Bytes([
            16, 0, 0, 0, // total_size
            0, 0, 0, 0, // reserved
            0, 0, 0, 0, // end tag type
            8, 0, 0, 0, // end tag size
        ]);
        let ptr = bytes.0.as_ptr();
        let bi = unsafe { BootInformation::load(ptr.cast()) };
        let bi = bi.unwrap();

        // compile time test
        accept(bi);
    }

    #[test]
    fn no_tags() {
        #[repr(C, align(8))]
        struct Bytes([u8; 16]);
        let bytes: Bytes = Bytes([
            16, 0, 0, 0, // total_size
            0, 0, 0, 0, // reserved
            0, 0, 0, 0, // end tag type
            8, 0, 0, 0, // end tag size
        ]);
        let ptr = bytes.0.as_ptr();
        let addr = ptr as usize;
        let bi = unsafe { BootInformation::load(ptr.cast()) };
        let bi = bi.unwrap();
        assert_eq!(addr, bi.start_address());
        assert_eq!(addr + bytes.0.len(), bi.end_address());
        assert_eq!(bytes.0.len(), bi.total_size());
        assert!(bi.elf_sections().is_none());
        assert!(bi.memory_map_tag().is_none());
        assert!(bi.module_tags().next().is_none());
        assert!(bi.boot_loader_name_tag().is_none());
        assert!(bi.command_line_tag().is_none());
    }

    #[test]
    #[should_panic]
    fn invalid_total_size() {
        #[repr(C, align(8))]
        struct Bytes([u8; 15]);
        let bytes: Bytes = Bytes([
            15, 0, 0, 0, // total_size
            0, 0, 0, 0, // reserved
            0, 0, 0, 0, // end tag type
            8, 0, 0, // end tag size
        ]);
        let ptr = bytes.0.as_ptr();
        let addr = ptr as usize;
        let bi = unsafe { BootInformation::load(ptr.cast()) };
        let bi = bi.unwrap();
        assert_eq!(addr, bi.start_address());
        assert_eq!(addr + bytes.0.len(), bi.end_address());
        assert_eq!(bytes.0.len(), bi.total_size());
        assert!(bi.elf_sections().is_none());
        assert!(bi.memory_map_tag().is_none());
        assert!(bi.module_tags().next().is_none());
        assert!(bi.boot_loader_name_tag().is_none());
        assert!(bi.command_line_tag().is_none());
    }

    #[test]
    #[should_panic]
    fn invalid_end_tag() {
        #[repr(C, align(8))]
        struct Bytes([u8; 16]);
        let bytes: Bytes = Bytes([
            16, 0, 0, 0, // total_size
            0, 0, 0, 0, // reserved
            0, 0, 0, 0, // end tag type
            9, 0, 0, 0, // end tag size
        ]);
        let ptr = bytes.0.as_ptr();
        let addr = ptr as usize;
        let bi = unsafe { BootInformation::load(ptr.cast()) };
        let bi = bi.unwrap();
        assert_eq!(addr, bi.start_address());
        assert_eq!(addr + bytes.0.len(), bi.end_address());
        assert_eq!(bytes.0.len(), bi.total_size());
        assert!(bi.elf_sections().is_none());
        assert!(bi.memory_map_tag().is_none());
        assert!(bi.module_tags().next().is_none());
        assert!(bi.boot_loader_name_tag().is_none());
        assert!(bi.command_line_tag().is_none());
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn name_tag() {
        #[repr(C, align(8))]
        struct Bytes([u8; 32]);
        let bytes: Bytes = Bytes([
            32, 0, 0, 0, // total_size
            0, 0, 0, 0, // reserved
            2, 0, 0, 0, // boot loader name tag type
            13, 0, 0, 0, // boot loader name tag size
            110, 97, 109, 101, // boot loader name 'name'
            0, 0, 0, 0, // boot loader name null + padding
            0, 0, 0, 0, // end tag type
            8, 0, 0, 0, // end tag size
        ]);
        let ptr = bytes.0.as_ptr();
        let addr = ptr as usize;
        let bi = unsafe { BootInformation::load(ptr.cast()) };
        let bi = bi.unwrap();
        assert_eq!(addr, bi.start_address());
        assert_eq!(addr + bytes.0.len(), bi.end_address());
        assert_eq!(bytes.0.len(), bi.total_size());
        assert!(bi.elf_sections().is_none());
        assert!(bi.memory_map_tag().is_none());
        assert!(bi.module_tags().next().is_none());
        assert_eq!(
            "name",
            bi.boot_loader_name_tag()
                .expect("tag must be present")
                .name()
                .expect("must be valid utf8")
        );
        assert!(bi.command_line_tag().is_none());
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn framebuffer_tag_rgb() {
        // direct RGB mode test:
        // taken from GRUB2 running in QEMU at
        // 1280x720 with 32bpp in BGRA format.
        #[repr(C, align(8))]
        struct Bytes([u8; 56]);
        let bytes: Bytes = Bytes([
            56, 0, 0, 0, // total size
            0, 0, 0, 0, // reserved
            8, 0, 0, 0, // framebuffer tag type
            40, 0, 0, 0, // framebuffer tag size
            0, 0, 0, 253, // framebuffer low dword of address
            0, 0, 0, 0, // framebuffer high dword of address
            0, 20, 0, 0, // framebuffer pitch
            0, 5, 0, 0, // framebuffer width
            208, 2, 0, 0, // framebuffer height
            32, 1, 0, 0, // framebuffer bpp, type, reserved word
            16, 8, 8, 8, // framebuffer red pos/size, green pos/size
            0, 8, 0, 0, // framebuffer blue pos/size, padding word
            0, 0, 0, 0, // end tag type
            8, 0, 0, 0, // end tag size
        ]);
        let ptr = bytes.0.as_ptr();
        let addr = ptr as usize;
        let bi = unsafe { BootInformation::load(ptr.cast()) };
        let bi = bi.unwrap();
        assert_eq!(addr, bi.start_address());
        assert_eq!(addr + bytes.0.len(), bi.end_address());
        assert_eq!(bytes.0.len(), bi.total_size());
        use framebuffer::{FramebufferField, FramebufferType};
        assert!(bi.framebuffer_tag().is_some());
        let fbi = bi
            .framebuffer_tag()
            .expect("Framebuffer info should be available")
            .expect("Framebuffer info type should be valid");
        assert_eq!(fbi.address(), 4244635648);
        assert_eq!(fbi.pitch(), 5120);
        assert_eq!(fbi.width(), 1280);
        assert_eq!(fbi.height(), 720);
        assert_eq!(fbi.bpp(), 32);
        assert_eq!(
            fbi.buffer_type().unwrap(),
            FramebufferType::RGB {
                red: FramebufferField {
                    position: 16,
                    size: 8,
                },
                green: FramebufferField {
                    position: 8,
                    size: 8,
                },
                blue: FramebufferField {
                    position: 0,
                    size: 8,
                },
            }
        );
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn framebuffer_tag_indexed() {
        // indexed mode test:
        // this is synthetic, as I can't get QEMU
        // to run in indexed color mode.
        #[repr(C, align(8))]
        struct Bytes([u8; 64]);
        let bytes: Bytes = Bytes([
            64, 0, 0, 0, // total size
            0, 0, 0, 0, // reserved
            8, 0, 0, 0, // framebuffer tag type
            48, 0, 0, 0, // framebuffer tag size
            0, 0, 0, 253, // framebuffer low dword of address
            0, 0, 0, 0, // framebuffer high dword of address
            0, 20, 0, 0, // framebuffer pitch
            0, 5, 0, 0, // framebuffer width
            208, 2, 0, 0, // framebuffer height
            32, 0, 0, 0, // framebuffer bpp, type, reserved word
            4, 0, 0, 0, // framebuffer palette length
            255, 0, 0, 0, // framebuffer palette
            255, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, // end tag type
            8, 0, 0, 0, // end tag size
        ]);
        let ptr = bytes.0.as_ptr();
        let addr = ptr as usize;
        let bi = unsafe { BootInformation::load(ptr.cast()) };
        let bi = bi.unwrap();
        assert_eq!(addr, bi.start_address());
        assert_eq!(addr + bytes.0.len(), bi.end_address());
        assert_eq!(bytes.0.len(), bi.total_size());
        use framebuffer::{FramebufferColor, FramebufferType};
        assert!(bi.framebuffer_tag().is_some());
        let fbi = bi
            .framebuffer_tag()
            .expect("Framebuffer info should be available")
            .expect("Framebuffer info type should be valid");
        assert_eq!(fbi.address(), 4244635648);
        assert_eq!(fbi.pitch(), 5120);
        assert_eq!(fbi.width(), 1280);
        assert_eq!(fbi.height(), 720);
        assert_eq!(fbi.bpp(), 32);
        match fbi.buffer_type().unwrap() {
            FramebufferType::Indexed { palette } => assert_eq!(
                palette,
                [
                    FramebufferColor {
                        red: 255,
                        green: 0,
                        blue: 0,
                    },
                    FramebufferColor {
                        red: 0,
                        green: 255,
                        blue: 0,
                    },
                    FramebufferColor {
                        red: 0,
                        green: 0,
                        blue: 255,
                    },
                    FramebufferColor {
                        red: 0,
                        green: 0,
                        blue: 0,
                    }
                ]
            ),
            _ => panic!("Expected indexed framebuffer type."),
        }
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn vbe_info_tag() {
        //Taken from GRUB2 running in QEMU.
        #[repr(C, align(8))]
        struct Bytes([u8; 800]);
        let bytes = Bytes([
            32, 3, 0, 0, // Total size.
            0, 0, 0, 0, // Reserved
            7, 0, 0, 0, // Tag type.
            16, 3, 0, 0, // Tag size.
            122, 65, 255, 255, // VBE mode, protected mode interface segment,
            0, 96, 79, 0, // protected mode interface offset, and length.
            86, 69, 83, 65, // "VESA" signature.
            0, 3, 220, 87, // VBE version, lower half of OEM string ptr,
            0, 192, 1, 0, // upper half of OEM string ptr, lower half of capabilities
            0, 0, 34, 128, // upper half of capabilities, lower half of vide mode ptr,
            0, 96, 0, 1, // upper half of video mode ptr, number of 64kb memory blocks
            0, 0, 240, 87, // OEM software revision, lower half of OEM vendor string ptr,
            0, 192, 3,
            88, // upper half of OEM vendor string ptr, lower half of OEM product string ptr,
            0, 192, 23,
            88, // upper half of OEM product string ptr, lower half of OEM revision string ptr,
            0, 192, 0, 1, // upper half of OEM revision string ptr.
            1, 1, 2, 1, // Reserved data....
            3, 1, 4, 1, 5, 1, 6, 1, 7, 1, 13, 1, 14, 1, 15, 1, 16, 1, 17, 1, 18, 1, 19, 1, 20, 1,
            21, 1, 22, 1, 23, 1, 24, 1, 25, 1, 26, 1, 27, 1, 28, 1, 29, 1, 30, 1, 31, 1, 64, 1, 65,
            1, 66, 1, 67, 1, 68, 1, 69, 1, 70, 1, 71, 1, 72, 1, 73, 1, 74, 1, 75, 1, 76, 1, 117, 1,
            118, 1, 119, 1, 120, 1, 121, 1, 122, 1, 123, 1, 124, 1, 125, 1, 126, 1, 127, 1, 128, 1,
            129, 1, 130, 1, 131, 1, 132, 1, 133, 1, 134, 1, 135, 1, 136, 1, 137, 1, 138, 1, 139, 1,
            140, 1, 141, 1, 142, 1, 143, 1, 144, 1, 145, 1, 146, 1, 0, 0, 1, 0, 2, 0, 3, 0, 4, 0,
            5, 0, 6, 0, 7, 0, 13, 0, 14, 0, 15, 0, 16, 0, 17, 0, 18, 0, 19, 0, 106, 0, 255, 255, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // Until Here
            187, 0, 7, 0, // Mode attributes, window A and B attributes
            64, 0, 64, 0, // Window granularity and size.
            0, 160, 0, 0, // Window A and B segments.
            186, 84, 0, 192, // Window relocation function pointer.
            0, 20, 0, 5, // Pitch, X resolution.
            32, 3, 8, 16, // Y resolution, X char size, Y char size.
            1, 32, 1, 6, // Number of planes, BPP, number of banks, memory model.
            0, 3, 1, 8, // Bank size, number of images, reserved, red mask size.
            16, 8, 8,
            8, // Red mask position, green mask size, green mask position, blue mask size,
            0, 8, 24,
            2, // blue mask position, reserved mask size, reserved mask position, color attributes.
            0, 0, 0, 253, // Frame buffer base address.
            0, 0, 0, 0, // Off screen memory offset.
            0, 0, 0, 20, // Off screen memory size, reserved data...
            0, 0, 8, 16, 8, 8, 8, 0, 8, 24, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, // Until here.
            0, 0, 0, 0, // End tag type.
            8, 0, 0, 0, // End tag size.
        ]);

        let ptr = bytes.0.as_ptr();
        let addr = ptr as usize;
        let bi = unsafe { BootInformation::load(ptr.cast()) };
        let bi = bi.unwrap();
        assert_eq!(addr, bi.start_address());
        assert_eq!(addr + bytes.0.len(), bi.end_address());
        assert_eq!(bytes.0.len(), bi.total_size());
        assert!(bi.vbe_info_tag().is_some());
        let vbe = bi.vbe_info_tag().unwrap();
        use vbe_info::*;

        assert_eq!({ vbe.mode }, 16762);
        assert_eq!({ vbe.interface_segment }, 65535);
        assert_eq!({ vbe.interface_offset }, 24576);
        assert_eq!({ vbe.interface_length }, 79);
        assert_eq!({ vbe.control_info.signature }, [86, 69, 83, 65]);
        assert_eq!({ vbe.control_info.version }, 768);
        assert_eq!({ vbe.control_info.oem_string_ptr }, 3221247964);
        assert_eq!(
            { vbe.control_info.capabilities },
            VBECapabilities::SWITCHABLE_DAC
        );
        assert_eq!({ vbe.control_info.mode_list_ptr }, 1610645538);
        assert_eq!({ vbe.control_info.total_memory }, 256);
        assert_eq!({ vbe.control_info.oem_software_revision }, 0);
        assert_eq!({ vbe.control_info.oem_vendor_name_ptr }, 3221247984);
        assert_eq!({ vbe.control_info.oem_product_name_ptr }, 3221248003);
        assert_eq!({ vbe.control_info.oem_product_revision_ptr }, 3221248023);
        assert!({ vbe.mode_info.mode_attributes }.contains(
            VBEModeAttributes::SUPPORTED
                | VBEModeAttributes::COLOR
                | VBEModeAttributes::GRAPHICS
                | VBEModeAttributes::NOT_VGA_COMPATIBLE
                | VBEModeAttributes::LINEAR_FRAMEBUFFER
        ));
        assert!(vbe.mode_info.window_a_attributes.contains(
            VBEWindowAttributes::RELOCATABLE
                | VBEWindowAttributes::READABLE
                | VBEWindowAttributes::WRITEABLE
        ));
        assert_eq!({ vbe.mode_info.window_granularity }, 64);
        assert_eq!({ vbe.mode_info.window_size }, 64);
        assert_eq!({ vbe.mode_info.window_a_segment }, 40960);
        assert_eq!({ vbe.mode_info.window_function_ptr }, 3221247162);
        assert_eq!({ vbe.mode_info.pitch }, 5120);
        assert_eq!({ vbe.mode_info.resolution }, (1280, 800));
        assert_eq!(vbe.mode_info.character_size, (8, 16));
        assert_eq!(vbe.mode_info.number_of_planes, 1);
        assert_eq!(vbe.mode_info.bpp, 32);
        assert_eq!(vbe.mode_info.number_of_banks, 1);
        assert_eq!(vbe.mode_info.memory_model, VBEMemoryModel::DirectColor);
        assert_eq!(vbe.mode_info.bank_size, 0);
        assert_eq!(vbe.mode_info.number_of_image_pages, 3);
        assert_eq!(
            vbe.mode_info.red_field,
            VBEField {
                position: 16,
                size: 8,
            }
        );
        assert_eq!(
            vbe.mode_info.green_field,
            VBEField {
                position: 8,
                size: 8,
            }
        );
        assert_eq!(
            vbe.mode_info.blue_field,
            VBEField {
                position: 0,
                size: 8,
            }
        );
        assert_eq!(
            vbe.mode_info.reserved_field,
            VBEField {
                position: 24,
                size: 8,
            }
        );
        assert_eq!(
            vbe.mode_info.direct_color_attributes,
            VBEDirectColorAttributes::RESERVED_USABLE
        );
        assert_eq!({ vbe.mode_info.framebuffer_base_ptr }, 4244635648);
        assert_eq!({ vbe.mode_info.offscreen_memory_offset }, 0);
        assert_eq!({ vbe.mode_info.offscreen_memory_size }, 0);
    }

    #[test]
    /// Compile time test for [`VBEInfoTag`].
    fn vbe_info_tag_size() {
        unsafe {
            // 16 for the start + 512 from `VBEControlInfo` + 256 from `VBEModeInfo`.
            core::mem::transmute::<[u8; 784], VBEInfoTag>([0u8; 784]);
        }
    }

    /// Tests to parse a MBI that was statically extracted from a test run with
    /// GRUB as bootloader.
    #[test]
    #[cfg_attr(miri, ignore)]
    fn grub2() {
        #[repr(C, align(8))]
        struct Bytes([u8; 960]);
        let mut bytes: Bytes = Bytes([
            192, 3, 0, 0, // total_size
            0, 0, 0, 0, // reserved
            1, 0, 0, 0, // boot command tag type
            9, 0, 0, 0, // boot command tag size
            0, 0, 0, 0, // boot command null + padding
            0, 0, 0, 0, // boot command padding
            2, 0, 0, 0, // boot loader name tag type
            26, 0, 0, 0, // boot loader name tag size
            71, 82, 85, 66, // boot loader name
            32, 50, 46, 48, // boot loader name
            50, 126, 98, 101, // boot loader name
            116, 97, 51, 45, // boot loader name
            53, 0, 0, 0, // boot loader name null + padding
            0, 0, 0, 0, // boot loader name padding
            10, 0, 0, 0, // APM tag type
            28, 0, 0, 0, // APM tag size
            2, 1, 0, 240, // APM version, cseg
            207, 212, 0, 0, // APM offset
            0, 240, 0, 240, // APM cseg_16, dseg
            3, 0, 240, 255, // APM flags, cseg_len
            240, 255, 240, 255, // APM cseg_16_len, dseg_len
            0, 0, 0, 0, // APM padding
            6, 0, 0, 0, // memory map tag type
            160, 0, 0, 0, // memory map tag size
            24, 0, 0, 0, // memory map entry_size
            0, 0, 0, 0, // memory map entry_version
            0, 0, 0, 0, // memory map entry 0 base_addr
            0, 0, 0, 0, // memory map entry 0 base_addr
            0, 252, 9, 0, // memory map entry 0 length
            0, 0, 0, 0, // memory map entry 0 length
            1, 0, 0, 0, // memory map entry 0 type
            0, 0, 0, 0, // memory map entry 0 reserved
            0, 252, 9, 0, // memory map entry 1 base_addr
            0, 0, 0, 0, // memory map entry 1 base_addr
            0, 4, 0, 0, // memory map entry 1 length
            0, 0, 0, 0, // memory map entry 1 length
            2, 0, 0, 0, // memory map entry 1 type
            0, 0, 0, 0, // memory map entry 1 reserved
            0, 0, 15, 0, // memory map entry 2 base_addr
            0, 0, 0, 0, // memory map entry 2 base_addr
            0, 0, 1, 0, // memory map entry 2 length
            0, 0, 0, 0, // memory map entry 2 length
            2, 0, 0, 0, // memory map entry 2 type
            0, 0, 0, 0, // memory map entry 2 reserved
            0, 0, 16, 0, // memory map entry 3 base_addr
            0, 0, 0, 0, // memory map entry 3 base_addr
            0, 0, 238, 7, // memory map entry 3 length
            0, 0, 0, 0, // memory map entry 3 length
            1, 0, 0, 0, // memory map entry 3 type
            0, 0, 0, 0, // memory map entry 3 reserved
            0, 0, 254, 7, // memory map entry 4 base_addr
            0, 0, 0, 0, // memory map entry 4 base_addr
            0, 0, 2, 0, // memory map entry 4 length
            0, 0, 0, 0, // memory map entry 4 length
            2, 0, 0, 0, // memory map entry 4 type
            0, 0, 0, 0, // memory map entry 4 reserved
            0, 0, 252, 255, // memory map entry 5 base_addr
            0, 0, 0, 0, // memory map entry 5 base_addr
            0, 0, 4, 0, // memory map entry 5 length
            0, 0, 0, 0, // memory map entry 5 length
            2, 0, 0, 0, // memory map entry 5 type
            0, 0, 0, 0, // memory map entry 5 reserved
            9, 0, 0, 0, // elf symbols tag type
            84, 2, 0, 0, // elf symbols tag size
            9, 0, 0, 0, // elf symbols num
            64, 0, 0, 0, // elf symbols entsize
            8, 0, 0, 0, // elf symbols shndx
            0, 0, 0, 0, // elf symbols entry 0 name
            0, 0, 0, 0, // elf symbols entry 0 type
            0, 0, 0, 0, // elf symbols entry 0 flags
            0, 0, 0, 0, // elf symbols entry 0 flags
            0, 0, 0, 0, // elf symbols entry 0 addr
            0, 0, 0, 0, // elf symbols entry 0 addr
            0, 0, 0, 0, // elf symbols entry 0 offset
            0, 0, 0, 0, // elf symbols entry 0 offset
            0, 0, 0, 0, // elf symbols entry 0 size
            0, 0, 0, 0, // elf symbols entry 0 size
            0, 0, 0, 0, // elf symbols entry 0 link
            0, 0, 0, 0, // elf symbols entry 0 info
            0, 0, 0, 0, // elf symbols entry 0 addralign
            0, 0, 0, 0, // elf symbols entry 0 addralign
            0, 0, 0, 0, // elf symbols entry 0 entsize
            0, 0, 0, 0, // elf symbols entry 0 entsize
            27, 0, 0, 0, // elf symbols entry 1 name
            1, 0, 0, 0, // elf symbols entry 1 type
            2, 0, 0, 0, // elf symbols entry 1 flags
            0, 0, 0, 0, // elf symbols entry 1 flags
            0, 0, 16, 0, // elf symbols entry 1 addr
            0, 128, 255, 255, // elf symbols entry 1 addr
            0, 16, 0, 0, // elf symbols entry 1 offset
            0, 0, 0, 0, // elf symbols entry 1 offset
            0, 48, 0, 0, // elf symbols entry 1 size
            0, 0, 0, 0, // elf symbols entry 1 size
            0, 0, 0, 0, // elf symbols entry 1 link
            0, 0, 0, 0, // elf symbols entry 1 info
            16, 0, 0, 0, // elf symbols entry 1 addralign
            0, 0, 0, 0, // elf symbols entry 1 addralign
            0, 0, 0, 0, // elf symbols entry 1 entsize
            0, 0, 0, 0, // elf symbols entry 1 entsize
            35, 0, 0, 0, // elf symbols entry 2 name
            1, 0, 0, 0, // elf symbols entry 2 type
            6, 0, 0, 0, // elf symbols entry 2 flags
            0, 0, 0, 0, // elf symbols entry 2 flags
            0, 48, 16, 0, // elf symbols entry 2 addr
            0, 128, 255, 255, // elf symbols entry 2 addr
            0, 64, 0, 0, // elf symbols entry 2 offset
            0, 0, 0, 0, // elf symbols entry 2 offset
            0, 144, 0, 0, // elf symbols entry 2 size
            0, 0, 0, 0, // elf symbols entry 2 size
            0, 0, 0, 0, // elf symbols entry 2 link
            0, 0, 0, 0, // elf symbols entry 2 info
            16, 0, 0, 0, // elf symbols entry 2 addralign
            0, 0, 0, 0, // elf symbols entry 2 addralign
            0, 0, 0, 0, // elf symbols entry 2 entsize
            0, 0, 0, 0, // elf symbols entry 2 entsize
            41, 0, 0, 0, // elf symbols entry 3 name
            1, 0, 0, 0, // elf symbols entry 3 type
            3, 0, 0, 0, // elf symbols entry 3 flags
            0, 0, 0, 0, // elf symbols entry 3 flags
            0, 192, 16, 0, // elf symbols entry 3 addr
            0, 128, 255, 255, // elf symbols entry 3 addr
            0, 208, 0, 0, // elf symbols entry 3 offset
            0, 0, 0, 0, // elf symbols entry 3 offset
            0, 32, 0, 0, // elf symbols entry 3 size
            0, 0, 0, 0, // elf symbols entry 3 size
            0, 0, 0, 0, // elf symbols entry 3 link
            0, 0, 0, 0, // elf symbols entry 3 info
            8, 0, 0, 0, // elf symbols entry 3 addralign
            0, 0, 0, 0, // elf symbols entry 3 addralign
            0, 0, 0, 0, // elf symbols entry 3 entsize
            0, 0, 0, 0, // elf symbols entry 3 entsize
            47, 0, 0, 0, // elf symbols entry 4 name
            8, 0, 0, 0, // elf symbols entry 4 type
            3, 0, 0, 0, // elf symbols entry 4 flags
            0, 0, 0, 0, // elf symbols entry 4 flags
            0, 224, 16, 0, // elf symbols entry 4 addr
            0, 128, 255, 255, // elf symbols entry 4 addr
            0, 240, 0, 0, // elf symbols entry 4 offset
            0, 0, 0, 0, // elf symbols entry 4 offset
            0, 80, 0, 0, // elf symbols entry 4 size
            0, 0, 0, 0, // elf symbols entry 4 size
            0, 0, 0, 0, // elf symbols entry 4 link
            0, 0, 0, 0, // elf symbols entry 4 info
            0, 16, 0, 0, // elf symbols entry 4 addralign
            0, 0, 0, 0, // elf symbols entry 4 addralign
            0, 0, 0, 0, // elf symbols entry 4 entsize
            0, 0, 0, 0, // elf symbols entry 4 entsize
            52, 0, 0, 0, // elf symbols entry 5 name
            1, 0, 0, 0, // elf symbols entry 5 type
            3, 0, 0, 0, // elf symbols entry 5 flags
            0, 0, 0, 0, // elf symbols entry 5 flags
            0, 48, 17, 0, // elf symbols entry 5 addr
            0, 128, 255, 255, // elf symbols entry 5 addr
            0, 240, 0, 0, // elf symbols entry 5 offset
            0, 0, 0, 0, // elf symbols entry 5 offset
            0, 0, 0, 0, // elf symbols entry 5 size
            0, 0, 0, 0, // elf symbols entry 5 size
            0, 0, 0, 0, // elf symbols entry 5 link
            0, 0, 0, 0, // elf symbols entry 5 info
            1, 0, 0, 0, // elf symbols entry 5 addralign
            0, 0, 0, 0, // elf symbols entry 5 addralign
            0, 0, 0, 0, // elf symbols entry 5 entsize
            0, 0, 0, 0, // elf symbols entry 5 entsize
            1, 0, 0, 0, // elf symbols entry 6 name
            2, 0, 0, 0, // elf symbols entry 6 type
            0, 0, 0, 0, // elf symbols entry 6 flags
            0, 0, 0, 0, // elf symbols entry 6 flags
            0, 48, 17, 0, // elf symbols entry 6 addr
            0, 0, 0, 0, // elf symbols entry 6 addr
            0, 240, 0, 0, // elf symbols entry 6 offset
            0, 0, 0, 0, // elf symbols entry 6 offset
            224, 43, 0, 0, // elf symbols entry 6 size
            0, 0, 0, 0, // elf symbols entry 6 size
            7, 0, 0, 0, // elf symbols entry 6 link
            102, 1, 0, 0, // elf symbols entry 6 info
            8, 0, 0, 0, // elf symbols entry 6 addralign
            0, 0, 0, 0, // elf symbols entry 6 addralign
            24, 0, 0, 0, // elf symbols entry 6 entsize
            0, 0, 0, 0, // elf symbols entry 6 entsize
            9, 0, 0, 0, // elf symbols entry 7 name
            3, 0, 0, 0, // elf symbols entry 7 type
            0, 0, 0, 0, // elf symbols entry 7 flags
            0, 0, 0, 0, // elf symbols entry 7 flags
            224, 91, 17, 0, // elf symbols entry 7 addr
            0, 0, 0, 0, // elf symbols entry 7 addr
            224, 27, 1, 0, // elf symbols entry 7 offset
            0, 0, 0, 0, // elf symbols entry 7 offset
            145, 55, 0, 0, // elf symbols entry 7 size
            0, 0, 0, 0, // elf symbols entry 7 size
            0, 0, 0, 0, // elf symbols entry 7 link
            0, 0, 0, 0, // elf symbols entry 7 info
            1, 0, 0, 0, // elf symbols entry 7 addralign
            0, 0, 0, 0, // elf symbols entry 7 addralign
            0, 0, 0, 0, // elf symbols entry 7 entsize
            0, 0, 0, 0, // elf symbols entry 7 entsize
            17, 0, 0, 0, // elf symbols entry 8 name
            3, 0, 0, 0, // elf symbols entry 8 type
            0, 0, 0, 0, // elf symbols entry 8 flags
            0, 0, 0, 0, // elf symbols entry 8 flags
            113, 147, 17, 0, // elf symbols entry 8 addr
            0, 0, 0, 0, // elf symbols entry 8 addr
            113, 83, 1, 0, // elf symbols entry 8 offset
            0, 0, 0, 0, // elf symbols entry 8 offset
            65, 0, 0, 0, // elf symbols entry 8 size
            0, 0, 0, 0, // elf symbols entry 8 size
            0, 0, 0, 0, // elf symbols entry 8 link
            0, 0, 0, 0, // elf symbols entry 8 info
            1, 0, 0, 0, // elf symbols entry 8 addralign
            0, 0, 0, 0, // elf symbols entry 8 addralign
            0, 0, 0, 0, // elf symbols entry 8 entsize
            0, 0, 0, 0, // elf symbols entry 8 entsize
            0, 0, 0, 0, // elf symbols padding
            4, 0, 0, 0, // basic memory tag type
            16, 0, 0, 0, // basic memory tag size
            127, 2, 0, 0, // basic memory mem_lower
            128, 251, 1, 0, // basic memory mem_upper
            5, 0, 0, 0, // BIOS boot device tag type
            20, 0, 0, 0, // BIOS boot device tag size
            224, 0, 0, 0, // BIOS boot device biosdev
            255, 255, 255, 255, // BIOS boot device partition
            255, 255, 255, 255, // BIOS boot device subpartition
            0, 0, 0, 0, // BIOS boot device padding
            8, 0, 0, 0, // framebuffer info tag type
            32, 0, 0, 0, // framebuffer info tag size
            0, 128, 11, 0, // framebuffer info framebuffer_addr
            0, 0, 0, 0, // framebuffer info framebuffer_addr
            160, 0, 0, 0, // framebuffer info framebuffer_pitch
            80, 0, 0, 0, // framebuffer info framebuffer_width
            25, 0, 0, 0, // framebuffer info framebuffer_height
            16, 2, 0, 0, // framebuffer info framebuffer_[bpp,type], reserved, color_info
            14, 0, 0, 0, // ACPI old tag type
            28, 0, 0, 0, // ACPI old tag size
            82, 83, 68, 32, // ACPI old
            80, 84, 82, 32, // ACPI old
            89, 66, 79, 67, // ACPI old
            72, 83, 32, 0, // ACPI old
            220, 24, 254, 7, // ACPI old
            0, 0, 0, 0, // ACPI old padding
            0, 0, 0, 0, // end tag type
            8, 0, 0, 0, // end tag size
        ]);
        #[repr(C, align(8))]
        struct StringBytes([u8; 65]);
        let string_bytes: StringBytes = StringBytes([
            0, 46, 115, 121, 109, 116, 97, 98, 0, 46, 115, 116, 114, 116, 97, 98, 0, 46, 115, 104,
            115, 116, 114, 116, 97, 98, 0, 46, 114, 111, 100, 97, 116, 97, 0, 46, 116, 101, 120,
            116, 0, 46, 100, 97, 116, 97, 0, 46, 98, 115, 115, 0, 46, 100, 97, 116, 97, 46, 114,
            101, 108, 46, 114, 111, 0,
        ]);
        let string_addr = string_bytes.0.as_ptr() as u64;
        for i in 0..8 {
            bytes.0[796 + i] = (string_addr >> (i * 8)) as u8;
        }
        let ptr = bytes.0.as_ptr();
        let addr = ptr as usize;
        let bi = unsafe { BootInformation::load(ptr.cast()) };
        let bi = bi.unwrap();
        test_grub2_boot_info(&bi, addr, string_addr, &bytes.0, &string_bytes.0);

        // Check that the MBI's debug output can be printed without SEGFAULT.
        // If this works, it is a good indicator than transitively a lot of
        // stuff works.
        println!("{bi:#?}");
    }

    /// Helper for [`grub2`].
    fn test_grub2_boot_info(
        bi: &BootInformation<&BootInformationInner>,
        addr: usize,
        string_addr: u64,
        bytes: &[u8],
        string_bytes: &[u8],
    ) {
        assert_eq!(addr, bi.start_address());
        assert_eq!(addr + bytes.len(), bi.end_address());
        assert_eq!(bytes.len(), bi.total_size());
        let mut es = bi.elf_sections().unwrap();
        let s1 = es.next().expect("Should have one more section");
        assert_eq!(".rodata", s1.name().expect("Should be valid utf-8"));
        assert_eq!(0xFFFF_8000_0010_0000, s1.start_address());
        assert_eq!(0xFFFF_8000_0010_3000, s1.end_address());
        assert_eq!(0x0000_0000_0000_3000, s1.size());
        assert_eq!(ElfSectionFlags::ALLOCATED, s1.flags());
        assert_eq!(ElfSectionType::ProgramSection, s1.section_type());
        let s2 = es.next().expect("Should have one more section");
        assert_eq!(".text", s2.name().expect("Should be valid utf-8"));
        assert_eq!(0xFFFF_8000_0010_3000, s2.start_address());
        assert_eq!(0xFFFF_8000_0010_C000, s2.end_address());
        assert_eq!(0x0000_0000_0000_9000, s2.size());
        assert_eq!(
            ElfSectionFlags::EXECUTABLE | ElfSectionFlags::ALLOCATED,
            s2.flags()
        );
        assert_eq!(ElfSectionType::ProgramSection, s2.section_type());
        let s3 = es.next().expect("Should have one more section");
        assert_eq!(".data", s3.name().expect("Should be valid utf-8"));
        assert_eq!(0xFFFF_8000_0010_C000, s3.start_address());
        assert_eq!(0xFFFF_8000_0010_E000, s3.end_address());
        assert_eq!(0x0000_0000_0000_2000, s3.size());
        assert_eq!(
            ElfSectionFlags::ALLOCATED | ElfSectionFlags::WRITABLE,
            s3.flags()
        );
        assert_eq!(ElfSectionType::ProgramSection, s3.section_type());
        let s4 = es.next().expect("Should have one more section");
        assert_eq!(".bss", s4.name().expect("Should be valid utf-8"));
        assert_eq!(0xFFFF_8000_0010_E000, s4.start_address());
        assert_eq!(0xFFFF_8000_0011_3000, s4.end_address());
        assert_eq!(0x0000_0000_0000_5000, s4.size());
        assert_eq!(
            ElfSectionFlags::ALLOCATED | ElfSectionFlags::WRITABLE,
            s4.flags()
        );
        assert_eq!(ElfSectionType::Uninitialized, s4.section_type());
        let s5 = es.next().expect("Should have one more section");
        assert_eq!(".data.rel.ro", s5.name().expect("Should be valid utf-8"));
        assert_eq!(0xFFFF_8000_0011_3000, s5.start_address());
        assert_eq!(0xFFFF_8000_0011_3000, s5.end_address());
        assert_eq!(0x0000_0000_0000_0000, s5.size());
        assert_eq!(
            ElfSectionFlags::ALLOCATED | ElfSectionFlags::WRITABLE,
            s5.flags()
        );
        assert_eq!(ElfSectionType::ProgramSection, s5.section_type());
        let s6 = es.next().expect("Should have one more section");
        assert_eq!(".symtab", s6.name().expect("Should be valid utf-8"));
        assert_eq!(0x0000_0000_0011_3000, s6.start_address());
        assert_eq!(0x0000_0000_0011_5BE0, s6.end_address());
        assert_eq!(0x0000_0000_0000_2BE0, s6.size());
        assert_eq!(ElfSectionFlags::empty(), s6.flags());
        assert_eq!(ElfSectionType::LinkerSymbolTable, s6.section_type());
        let s7 = es.next().expect("Should have one more section");
        assert_eq!(".strtab", s7.name().expect("Should be valid utf-8"));
        assert_eq!(0x0000_0000_0011_5BE0, s7.start_address());
        assert_eq!(0x0000_0000_0011_9371, s7.end_address());
        assert_eq!(0x0000_0000_0000_3791, s7.size());
        assert_eq!(ElfSectionFlags::empty(), s7.flags());
        assert_eq!(ElfSectionType::StringTable, s7.section_type());
        let s8 = es.next().expect("Should have one more section");
        assert_eq!(".shstrtab", s8.name().expect("Should be valid utf-8"));
        assert_eq!(string_addr, s8.start_address());
        assert_eq!(string_addr + string_bytes.len() as u64, s8.end_address());
        assert_eq!(string_bytes.len() as u64, s8.size());
        assert_eq!(ElfSectionFlags::empty(), s8.flags());
        assert_eq!(ElfSectionType::StringTable, s8.section_type());
        assert!(es.next().is_none());
        let mut mm = bi
            .memory_map_tag()
            .unwrap()
            .memory_areas()
            .iter()
            .filter(|area| area.typ() == MemoryAreaType::Available);
        let mm1 = mm.next().unwrap();
        assert_eq!(0x00000000, mm1.start_address());
        assert_eq!(0x009_FC00, mm1.end_address());
        assert_eq!(0x009_FC00, mm1.size());
        assert_eq!(MemoryAreaType::Available, mm1.typ());
        let mm2 = mm.next().unwrap();
        assert_eq!(0x010_0000, mm2.start_address());
        assert_eq!(0x7FE_0000, mm2.end_address());
        assert_eq!(0x7EE_0000, mm2.size());
        assert_eq!(MemoryAreaType::Available, mm2.typ());
        assert!(mm.next().is_none());

        // Test the RSDP tag
        let rsdp_old = bi.rsdp_v1_tag().unwrap();
        assert_eq!("RSD PTR ", rsdp_old.signature().unwrap());
        assert!(rsdp_old.checksum_is_valid());
        assert_eq!("BOCHS ", rsdp_old.oem_id().unwrap());
        assert_eq!(0, rsdp_old.revision());
        assert_eq!(0x7FE18DC, rsdp_old.rsdt_address());

        assert!(bi.module_tags().next().is_none());
        assert_eq!(
            "GRUB 2.02~beta3-5",
            bi.boot_loader_name_tag()
                .expect("tag must be present")
                .name()
                .expect("must be valid utf-8")
        );
        assert_eq!(
            "",
            bi.command_line_tag()
                .expect("tag must present")
                .cmdline()
                .expect("must be valid utf-8")
        );

        // Test the Framebuffer tag
        let fbi = bi
            .framebuffer_tag()
            .expect("Framebuffer info should be available")
            .expect("Framebuffer info type should be valid");
        assert_eq!(fbi.address(), 753664);
        assert_eq!(fbi.pitch(), 160);
        assert_eq!(fbi.width(), 80);
        assert_eq!(fbi.height(), 25);
        assert_eq!(fbi.bpp(), 16);
        assert_eq!(fbi.buffer_type(), Ok(FramebufferType::Text));
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn elf_sections() {
        #[repr(C, align(8))]
        struct Bytes([u8; 168]);
        let mut bytes: Bytes = Bytes([
            168, 0, 0, 0, // total_size
            0, 0, 0, 0, // reserved
            9, 0, 0, 0, // elf symbols tag type
            20, 2, 0, 0, // elf symbols tag size
            2, 0, 0, 0, // elf symbols num
            64, 0, 0, 0, // elf symbols entsize
            1, 0, 0, 0, // elf symbols shndx
            0, 0, 0, 0, // elf symbols entry 0 name
            0, 0, 0, 0, // elf symbols entry 0 type
            0, 0, 0, 0, // elf symbols entry 0 flags
            0, 0, 0, 0, // elf symbols entry 0 flags
            0, 0, 0, 0, // elf symbols entry 0 addr
            0, 0, 0, 0, // elf symbols entry 0 addr
            0, 0, 0, 0, // elf symbols entry 0 offset
            0, 0, 0, 0, // elf symbols entry 0 offset
            0, 0, 0, 0, // elf symbols entry 0 size
            0, 0, 0, 0, // elf symbols entry 0 size
            0, 0, 0, 0, // elf symbols entry 0 link
            0, 0, 0, 0, // elf symbols entry 0 info
            0, 0, 0, 0, // elf symbols entry 0 addralign
            0, 0, 0, 0, // elf symbols entry 0 addralign
            0, 0, 0, 0, // elf symbols entry 0 entsize
            0, 0, 0, 0, // elf symbols entry 0 entsize
            1, 0, 0, 0, // elf symbols entry 1 name
            3, 0, 0, 0, // elf symbols entry 1 type
            0, 0, 0, 0, // elf symbols entry 1 flags
            0, 0, 0, 0, // elf symbols entry 1 flags
            255, 255, 255, 255, // elf symbols entry 1 addr
            255, 255, 255, 255, // elf symbols entry 1 addr
            113, 83, 1, 0, // elf symbols entry 1 offset
            0, 0, 0, 0, // elf symbols entry 1 offset
            11, 0, 0, 0, // elf symbols entry 1 size
            0, 0, 0, 0, // elf symbols entry 1 size
            0, 0, 0, 0, // elf symbols entry 1 link
            0, 0, 0, 0, // elf symbols entry 1 info
            1, 0, 0, 0, // elf symbols entry 1 addralign
            0, 0, 0, 0, // elf symbols entry 1 addralign
            0, 0, 0, 0, // elf symbols entry 1 entsize
            0, 0, 0, 0, // elf symbols entry 1 entsize
            0, 0, 0, 0, // elf symbols padding
            0, 0, 0, 0, // end tag type
            8, 0, 0, 0, // end tag size
        ]);
        #[repr(C, align(8))]
        struct StringBytes([u8; 11]);
        let string_bytes: StringBytes =
            StringBytes([0, 46, 115, 104, 115, 116, 114, 116, 97, 98, 0]);
        let string_addr = string_bytes.0.as_ptr() as u64;
        for i in 0..8 {
            let offset = 108;
            assert_eq!(255, bytes.0[offset + i]);
            bytes.0[offset + i] = (string_addr >> (i * 8)) as u8;
        }
        let ptr = bytes.0.as_ptr();
        let addr = ptr as usize;
        let bi = unsafe { BootInformation::load(ptr.cast()) };
        let bi = bi.unwrap();
        assert_eq!(addr, bi.start_address());
        assert_eq!(addr + bytes.0.len(), bi.end_address());
        assert_eq!(bytes.0.len(), bi.total_size());
        let mut es = bi.elf_sections().unwrap();
        let s1 = es.next().expect("Should have one more section");
        assert_eq!(".shstrtab", s1.name().expect("Should be valid utf-8"));
        assert_eq!(string_addr, s1.start_address());
        assert_eq!(string_addr + string_bytes.0.len() as u64, s1.end_address());
        assert_eq!(string_bytes.0.len() as u64, s1.size());
        assert_eq!(ElfSectionFlags::empty(), s1.flags());
        assert_eq!(ElfSectionType::StringTable, s1.section_type());
        assert!(es.next().is_none());
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn efi_memory_map() {
        #[repr(C, align(8))]
        struct Bytes([u8; 72]);
        // test that the EFI memory map is detected.
        let bytes: Bytes = Bytes([
            72, 0, 0, 0, // size
            0, 0, 0, 0, // reserved
            17, 0, 0, 0, // EFI memory map type
            56, 0, 0, 0, // EFI memory map size
            48, 0, 0, 0, // EFI descriptor size
            1, 0, 0, 0, // EFI descriptor version, don't think this matters.
            7, 0, 0, 0, // Type: EfiConventionalMemory
            0, 0, 0, 0, // Padding
            0, 0, 16, 0, // Physical Address: should be 0x100000
            0, 0, 0, 0, // Extension of physical address.
            0, 0, 16, 0, // Virtual Address: should be 0x100000
            0, 0, 0, 0, // Extension of virtual address.
            4, 0, 0, 0, // 4 KiB Pages: 16 KiB
            0, 0, 0, 0, // Extension of pages
            0, 0, 0, 0, // Attributes of this memory range.
            0, 0, 0, 0, // Extension of attributes
            0, 0, 0, 0, // end tag type.
            8, 0, 0, 0, // end tag size.
        ]);
        let ptr = bytes.0.as_ptr();
        let addr = ptr as usize;
        let bi = unsafe { BootInformation::load(ptr.cast()) };
        let bi = bi.unwrap();
        assert_eq!(addr, bi.start_address());
        assert_eq!(addr + bytes.0.len(), bi.end_address());
        assert_eq!(bytes.0.len(), bi.total_size());
        let efi_memory_map = bi.efi_memory_map_tag().unwrap();
        let mut efi_mmap_iter = efi_memory_map.memory_areas();
        let desc = efi_mmap_iter.next().unwrap();
        assert_eq!(desc.phys_start, 0x100000);
        assert_eq!(desc.page_count, 4);
        assert_eq!(desc.ty, EFIMemoryAreaType::CONVENTIONAL);
        // test that the EFI memory map is not detected if the boot services
        // are not exited.
        struct Bytes2([u8; 80]);
        let bytes2: Bytes2 = Bytes2([
            80, 0, 0, 0, // size
            0, 0, 0, 0, // reserved
            17, 0, 0, 0, // EFI memory map type
            56, 0, 0, 0, // EFI memory map size
            48, 0, 0, 0, // EFI descriptor size
            1, 0, 0, 0, // EFI descriptor version, don't think this matters.
            7, 0, 0, 0, // Type: EfiConventionalMemory
            0, 0, 0, 0, // Padding
            0, 0, 16, 0, // Physical Address: should be 0x100000
            0, 0, 0, 0, // Extension of physical address.
            0, 0, 16, 0, // Virtual Address: should be 0x100000
            0, 0, 0, 0, // Extension of virtual address.
            4, 0, 0, 0, // 4 KiB Pages: 16 KiB
            0, 0, 0, 0, // Extension of pages
            0, 0, 0, 0, // Attributes of this memory range.
            0, 0, 0, 0, // Extension of attributes
            18, 0, 0, 0, // Tag ExitBootServices not terminated.
            8, 0, 0, 0, // Tag ExitBootServices size.
            0, 0, 0, 0, // end tag type.
            8, 0, 0, 0, // end tag size.
        ]);
        let bi = unsafe { BootInformation::load(bytes2.0.as_ptr().cast()) };
        let bi = bi.unwrap();
        let efi_mmap = bi.efi_memory_map_tag();
        assert!(efi_mmap.is_none());
    }

    #[test]
    #[cfg(feature = "unstable")]
    /// This test succeeds if it compiles.
    fn mbi_load_error_implements_error() {
        fn consumer<E: core::error::Error>(_e: E) {}
        consumer(MbiLoadError::IllegalAddress)
    }

    /// Example for a custom tag.
    #[test]
    #[cfg_attr(miri, ignore)]
    fn get_custom_tag_from_mbi() {
        #[repr(C, align(8))]
        struct CustomTag {
            tag: TagTypeId,
            size: u32,
            foo: u32,
        }

        impl TagTrait for CustomTag {
            const ID: TagType = TagType::Custom(0x1337);

            fn dst_size(_base_tag: &Tag) {}
        }

        #[repr(C, align(8))]
        struct AlignedBytes([u8; 32]);
        // Raw bytes of a MBI that only contains the custom tag.
        let bytes: AlignedBytes = AlignedBytes([
            32,
            0,
            0,
            0, // end: total size
            0,
            0,
            0,
            0, // end: padding; end of multiboot2 boot information begin
            CustomTag::ID.val().to_le_bytes()[0],
            CustomTag::ID.val().to_le_bytes()[1],
            CustomTag::ID.val().to_le_bytes()[2],
            CustomTag::ID.val().to_le_bytes()[3], // end: my custom tag id
            12,
            0,
            0,
            0, // end: tag size
            42,
            0,
            0,
            0,
            0,
            0,
            0,
            0, // 8 byte padding
            0,
            0,
            0,
            0, // end: end tag type
            8,
            0,
            0,
            0, // end: end tag size
        ]);
        let ptr = bytes.0.as_ptr();
        let addr = ptr as usize;
        let bi = unsafe { BootInformation::load(ptr.cast()) };
        let bi = bi.unwrap();
        assert_eq!(addr, bi.start_address());
        assert_eq!(addr + bytes.0.len(), bi.end_address());
        assert_eq!(bytes.0.len(), bi.total_size());

        let tag = bi.get_tag::<CustomTag>().unwrap();
        assert_eq!(tag.foo, 42);
    }

    /// Example for a custom DST tag.
    #[test]
    #[cfg_attr(miri, ignore)]
    fn get_custom_dst_tag_from_mbi() {
        #[repr(C)]
        #[derive(crate::Pointee)]
        struct CustomTag {
            tag: TagTypeId,
            size: u32,
            name: [u8],
        }

        impl CustomTag {
            fn name(&self) -> Result<&str, StringError> {
                Tag::parse_slice_as_string(&self.name)
            }
        }

        impl TagTrait for CustomTag {
            const ID: TagType = TagType::Custom(0x1337);

            fn dst_size(base_tag: &Tag) -> usize {
                // The size of the sized portion of the command line tag.
                let tag_base_size = 8;
                assert!(base_tag.size >= 8);
                base_tag.size as usize - tag_base_size
            }
        }

        #[repr(C, align(8))]
        struct AlignedBytes([u8; 32]);
        // Raw bytes of a MBI that only contains the custom tag.
        let bytes: AlignedBytes = AlignedBytes([
            32,
            0,
            0,
            0, // end: total size
            0,
            0,
            0,
            0, // end: padding; end of multiboot2 boot information begin
            CustomTag::ID.val().to_le_bytes()[0],
            CustomTag::ID.val().to_le_bytes()[1],
            CustomTag::ID.val().to_le_bytes()[2],
            CustomTag::ID.val().to_le_bytes()[3], // end: my custom tag id
            14,
            0,
            0,
            0, // end: tag size
            b'h',
            b'e',
            b'l',
            b'l',
            b'o',
            b'\0',
            0,
            0, // 2 byte padding
            0,
            0,
            0,
            0, // end: end tag type
            8,
            0,
            0,
            0, // end: end tag size
        ]);
        let ptr = bytes.0.as_ptr();
        let addr = ptr as usize;
        let bi = unsafe { BootInformation::load(ptr.cast()) };
        let bi = bi.unwrap();
        assert_eq!(addr, bi.start_address());
        assert_eq!(addr + bytes.0.len(), bi.end_address());
        assert_eq!(bytes.0.len(), bi.total_size());

        let tag = bi.get_tag::<CustomTag>().unwrap();
        assert_eq!(tag.name(), Ok("hello"));
    }

    /// Tests that `get_tag` can consume multiple types that implement `Into<TagTypeId>`
    #[test]
    #[cfg_attr(miri, ignore)]
    fn get_tag_into_variants() {
        #[repr(C, align(8))]
        struct Bytes([u8; 32]);
        let bytes: Bytes = Bytes([
            32,
            0,
            0,
            0, // total_size
            0,
            0,
            0,
            0, // reserved
            TagType::Cmdline.val().to_le_bytes()[0],
            TagType::Cmdline.val().to_le_bytes()[1],
            TagType::Cmdline.val().to_le_bytes()[2],
            TagType::Cmdline.val().to_le_bytes()[3],
            13,
            0,
            0,
            0, // tag size
            110,
            97,
            109,
            101, // ASCII string 'name'
            0,
            0,
            0,
            0, // null byte + padding
            0,
            0,
            0,
            0, // end tag type
            8,
            0,
            0,
            0, // end tag size
        ]);

        let ptr = bytes.0.as_ptr();
        let bi = unsafe { BootInformation::load(ptr.cast()) };
        let bi = bi.unwrap();

        let _tag = bi.get_tag::<CommandLineTag>().unwrap();
    }
}
