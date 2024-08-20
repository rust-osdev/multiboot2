//! Module for [`MemoryMapTag`], [`EFIMemoryMapTag`] and [`BasicMemoryInfoTag`]
//! and corresponding helper types.

pub use uefi_raw::table::boot::MemoryAttribute as EFIMemoryAttribute;
pub use uefi_raw::table::boot::MemoryDescriptor as EFIMemoryDesc;
pub use uefi_raw::table::boot::MemoryType as EFIMemoryAreaType;

use crate::tag::TagHeader;
use crate::{TagType, TagTypeId};
use core::fmt::{Debug, Formatter};
use core::marker::PhantomData;
use core::mem;
use multiboot2_common::{MaybeDynSized, Tag};
#[cfg(feature = "builder")]
use {alloc::boxed::Box, core::slice, multiboot2_common::new_boxed};

/// This tag provides an initial host memory map (legacy boot, not UEFI).
///
/// The map provided is guaranteed to list all standard RAM that should be
/// available for normal use. This type however includes the regions occupied
/// by kernel, mbi, segments and modules. Kernel must take care not to
/// overwrite these regions.
///
/// This tag may not be provided by some boot loaders on EFI platforms if EFI
/// boot services are enabled and available for the loaded image (The EFI boot
/// services tag may exist in the Multiboot2 boot information structure).
#[derive(ptr_meta::Pointee, Debug, PartialEq, Eq)]
#[repr(C, align(8))]
pub struct MemoryMapTag {
    header: TagHeader,
    entry_size: u32,
    entry_version: u32,
    areas: [MemoryArea],
}

impl MemoryMapTag {
    /// Constructs a new tag.
    #[cfg(feature = "builder")]
    #[must_use]
    pub fn new(areas: &[MemoryArea]) -> Box<Self> {
        let header = TagHeader::new(Self::ID, 0);
        let entry_size = (mem::size_of::<MemoryArea>() as u32).to_ne_bytes();
        let entry_version = 0_u32.to_ne_bytes();
        let areas = {
            let ptr = areas.as_ptr().cast::<u8>();
            let len = mem::size_of_val(areas);
            unsafe { slice::from_raw_parts(ptr, len) }
        };
        new_boxed(header, &[&entry_size, &entry_version, areas])
    }

    /// Returns the entry size.
    #[must_use]
    pub const fn entry_size(&self) -> u32 {
        self.entry_size
    }

    /// Returns the entry version.
    #[must_use]
    pub const fn entry_version(&self) -> u32 {
        self.entry_version
    }

    /// Return the slice of the provided [`MemoryArea`]s.
    ///
    /// Usually, this should already reflect the memory consumed by the
    /// code running this.
    #[must_use]
    pub fn memory_areas(&self) -> &[MemoryArea] {
        // If this ever fails, we need to model this differently in this crate.
        assert_eq!(self.entry_size as usize, mem::size_of::<MemoryArea>());
        &self.areas
    }
}

impl MaybeDynSized for MemoryMapTag {
    type Header = TagHeader;

    const BASE_SIZE: usize = mem::size_of::<TagHeader>() + 2 * mem::size_of::<u32>();

    fn dst_len(header: &TagHeader) -> usize {
        assert!(header.size as usize >= Self::BASE_SIZE);
        let size = header.size as usize - Self::BASE_SIZE;
        assert_eq!(size % mem::size_of::<MemoryArea>(), 0);
        size / mem::size_of::<MemoryArea>()
    }
}

impl Tag for MemoryMapTag {
    type IDType = TagType;

    const ID: TagType = TagType::Mmap;
}

/// A descriptor for an available or taken area of physical memory.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct MemoryArea {
    base_addr: u64,
    length: u64,
    typ: MemoryAreaTypeId,
    _reserved: u32,
}

impl MemoryArea {
    /// Create a new MemoryArea.
    pub fn new(base_addr: u64, length: u64, typ: impl Into<MemoryAreaTypeId>) -> Self {
        Self {
            base_addr,
            length,
            typ: typ.into(),
            _reserved: 0,
        }
    }

    /// The start address of the memory region.
    #[must_use]
    pub const fn start_address(&self) -> u64 {
        self.base_addr
    }

    /// The end address of the memory region.
    #[must_use]
    pub const fn end_address(&self) -> u64 {
        self.base_addr + self.length
    }

    /// The size, in bytes, of the memory region.
    #[must_use]
    pub const fn size(&self) -> u64 {
        self.length
    }

    /// The type of the memory region.
    #[must_use]
    pub const fn typ(&self) -> MemoryAreaTypeId {
        self.typ
    }
}

impl Debug for MemoryArea {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MemoryArea")
            .field("base_addr", &self.base_addr)
            .field("length", &self.length)
            .field("typ", &self.typ)
            .finish()
    }
}

/// ABI-friendly version of [`MemoryAreaType`].
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct MemoryAreaTypeId(u32);

impl From<u32> for MemoryAreaTypeId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<MemoryAreaTypeId> for u32 {
    fn from(value: MemoryAreaTypeId) -> Self {
        value.0
    }
}

impl Debug for MemoryAreaTypeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mt = MemoryAreaType::from(*self);
        Debug::fmt(&mt, f)
    }
}

/// Abstraction over defined memory types for the memory map as well as custom
/// ones. Types 1 to 5 are defined in the Multiboot2 spec and correspond to the
/// entry types of e820 memory maps.
///
/// This is not binary compatible with the Multiboot2 spec. Please use
/// [`MemoryAreaTypeId`] instead.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MemoryAreaType {
    /// Available memory free to be used by the OS.
    Available, /* 1 */

    /// A reserved area that must not be used.
    Reserved, /* 2, */

    /// Usable memory holding ACPI information.
    AcpiAvailable, /* 3, */

    /// Reserved memory which needs to be preserved on hibernation.
    /// Also called NVS in spec, which stands for "Non-Volatile Sleep/Storage",
    /// which is part of ACPI specification.
    ReservedHibernate, /* 4, */

    /// Memory which is occupied by defective RAM modules.
    Defective, /* = 5, */

    /// Custom memory map type.
    Custom(u32),
}

impl From<MemoryAreaTypeId> for MemoryAreaType {
    fn from(value: MemoryAreaTypeId) -> Self {
        match value.0 {
            1 => Self::Available,
            2 => Self::Reserved,
            3 => Self::AcpiAvailable,
            4 => Self::ReservedHibernate,
            5 => Self::Defective,
            val => Self::Custom(val),
        }
    }
}

impl From<MemoryAreaType> for MemoryAreaTypeId {
    fn from(value: MemoryAreaType) -> Self {
        let integer = match value {
            MemoryAreaType::Available => 1,
            MemoryAreaType::Reserved => 2,
            MemoryAreaType::AcpiAvailable => 3,
            MemoryAreaType::ReservedHibernate => 4,
            MemoryAreaType::Defective => 5,
            MemoryAreaType::Custom(val) => val,
        };
        integer.into()
    }
}

impl PartialEq<MemoryAreaType> for MemoryAreaTypeId {
    fn eq(&self, other: &MemoryAreaType) -> bool {
        let val: Self = (*other).into();
        let val: u32 = val.0;
        self.0.eq(&val)
    }
}

impl PartialEq<MemoryAreaTypeId> for MemoryAreaType {
    fn eq(&self, other: &MemoryAreaTypeId) -> bool {
        let val: MemoryAreaTypeId = (*self).into();
        let val: u32 = val.0;
        other.0.eq(&val)
    }
}

/// Basic memory info tag.
///
/// This tag includes "basic memory information". This means (legacy) lower and
/// upper memory: In Real Mode (modeled after the 8086), only the first 1MB of
/// memory is accessible. Typically, the region between 640KB and 1MB is not
/// freely usable, because it is used for memory-mapped IO, for instance. The
/// term “lower memory” refers to those first 640KB of memory that are freely
/// usable for an application in Real Mode. “Upper memory” then refers to the
/// next freely usable chunk of memory, starting at 1MB up to about 10MB, in
/// practice. This is the memory an application running on a 286 (which had a
/// 24-bit address bus) could use, historically.
///
/// Nowadays, much bigger chunks of continuous memory are available at higher
/// addresses, but the Multiboot standard still references those two terms.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct BasicMemoryInfoTag {
    header: TagHeader,
    memory_lower: u32,
    memory_upper: u32,
}

impl BasicMemoryInfoTag {
    /// Constructs a new tag.
    #[must_use]
    pub fn new(memory_lower: u32, memory_upper: u32) -> Self {
        Self {
            header: TagHeader::new(Self::ID, mem::size_of::<Self>().try_into().unwrap()),
            memory_lower,
            memory_upper,
        }
    }

    #[must_use]
    /// Returns the lower memory bound.
    pub const fn memory_lower(&self) -> u32 {
        self.memory_lower
    }

    #[must_use]
    /// Returns the upper memory bound.
    pub const fn memory_upper(&self) -> u32 {
        self.memory_upper
    }
}

impl MaybeDynSized for BasicMemoryInfoTag {
    type Header = TagHeader;

    const BASE_SIZE: usize = mem::size_of::<Self>();

    fn dst_len(_: &TagHeader) {}
}

impl Tag for BasicMemoryInfoTag {
    type IDType = TagType;

    const ID: TagType = TagType::BasicMeminfo;
}

/// EFI memory map tag. The embedded [`EFIMemoryDesc`]s follows the EFI
/// specification.
#[derive(ptr_meta::Pointee, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct EFIMemoryMapTag {
    header: TagHeader,
    /// Most likely a little more than the size of a [`EFIMemoryDesc`].
    /// This is always the reference, and `size_of` never.
    /// See <https://github.com/tianocore/edk2/blob/7142e648416ff5d3eac6c6d607874805f5de0ca8/MdeModulePkg/Core/PiSmmCore/Page.c#L1059>.
    desc_size: u32,
    /// Version of the tag. The spec leaves it open to extend the memory
    /// descriptor in the future. However, this never happened so far.
    /// At the moment, only version "1" is supported.
    desc_version: u32,
    /// Contains the UEFI memory map.
    ///
    /// To follow the UEFI spec and to allow extendability for future UEFI
    /// revisions, the length is a multiple of `desc_size` and not a multiple
    /// of `size_of::<EfiMemoryDescriptor>()`.
    ///
    /// This tag is properly `align_of::<EFIMemoryDesc>` aligned, if the tag
    /// itself is also 8 byte aligned, which every sane MBI guarantees.
    memory_map: [u8],
}

impl EFIMemoryMapTag {
    /// Create a new EFI memory map tag with the given memory descriptors.
    #[cfg(feature = "builder")]
    #[must_use]
    pub fn new_from_descs(descs: &[EFIMemoryDesc]) -> Box<Self> {
        let efi_mmap = {
            let ptr = descs.as_ptr().cast::<u8>();
            let len = mem::size_of_val(descs);
            unsafe { slice::from_raw_parts(ptr, len) }
        };

        Self::new_from_map(
            mem::size_of::<EFIMemoryDesc>() as u32,
            EFIMemoryDesc::VERSION,
            efi_mmap,
        )
    }

    /// Create a new EFI memory map tag from the given EFI memory map.
    #[cfg(feature = "builder")]
    #[must_use]
    pub fn new_from_map(desc_size: u32, desc_version: u32, efi_mmap: &[u8]) -> Box<Self> {
        let header = TagHeader::new(Self::ID, 0);
        assert_ne!(desc_size, 0);
        let desc_size = desc_size.to_ne_bytes();
        let desc_version = desc_version.to_ne_bytes();
        new_boxed(header, &[&desc_size, &desc_version, efi_mmap])
    }

    /// Returns an iterator over the provided memory areas.
    ///
    /// Usually, this should already reflect the memory consumed by the
    /// code running this.
    #[must_use]
    pub fn memory_areas(&self) -> EFIMemoryAreaIter {
        // If this ever fails, this needs to be refactored in a joint-effort
        // with the uefi-rs project to have all corresponding typings.
        assert_eq!(self.desc_version, EFIMemoryDesc::VERSION);
        assert_eq!(
            self.memory_map
                .as_ptr()
                .align_offset(mem::align_of::<EFIMemoryDesc>()),
            0
        );

        EFIMemoryAreaIter::new(self)
    }
}

impl Debug for EFIMemoryMapTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("EFIMemoryMapTag")
            .field("typ", &self.header.typ)
            .field("size", &self.header.size)
            .field("desc_size", &self.desc_size)
            .field("buf", &self.memory_map.as_ptr())
            .field("buf_len", &self.memory_map.len())
            .field("entries", &self.memory_areas().len())
            .finish()
    }
}

impl MaybeDynSized for EFIMemoryMapTag {
    type Header = TagHeader;

    const BASE_SIZE: usize = mem::size_of::<TagTypeId>() + 3 * mem::size_of::<u32>();

    fn dst_len(header: &TagHeader) -> usize {
        assert!(header.size as usize >= Self::BASE_SIZE);
        header.size as usize - Self::BASE_SIZE
    }
}

impl Tag for EFIMemoryMapTag {
    type IDType = TagType;

    const ID: TagType = TagType::EfiMmap;
}

/// An iterator over the EFI memory areas emitting [`EFIMemoryDesc`] items.
#[derive(Clone, Debug)]
pub struct EFIMemoryAreaIter<'a> {
    mmap_tag: &'a EFIMemoryMapTag,
    i: usize,
    entries: usize,
    phantom: PhantomData<&'a EFIMemoryDesc>,
}

impl<'a> EFIMemoryAreaIter<'a> {
    fn new(mmap_tag: &'a EFIMemoryMapTag) -> Self {
        let desc_size = mmap_tag.desc_size as usize;
        let mmap_len = mmap_tag.memory_map.len();
        assert_eq!(mmap_len % desc_size, 0, "memory map length must be a multiple of `desc_size` by definition. The MBI seems to be corrupt.");
        Self {
            mmap_tag,
            i: 0,
            entries: mmap_len / desc_size,
            phantom: PhantomData,
        }
    }
}

impl<'a> Iterator for EFIMemoryAreaIter<'a> {
    type Item = &'a EFIMemoryDesc;
    fn next(&mut self) -> Option<&'a EFIMemoryDesc> {
        if self.i >= self.entries {
            return None;
        }

        let desc = unsafe {
            self.mmap_tag
                .memory_map
                .as_ptr()
                .add(self.i * self.mmap_tag.desc_size as usize)
                .cast::<EFIMemoryDesc>()
                .as_ref()
                .unwrap()
        };

        self.i += 1;

        Some(desc)
    }
}

impl<'a> ExactSizeIterator for EFIMemoryAreaIter<'a> {
    fn len(&self) -> usize {
        self.entries
    }
}

#[cfg(all(test, feature = "builder"))]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn test_create_old_mmap() {
        let _mmap = MemoryMapTag::new(&[]);
        let mmap = MemoryMapTag::new(&[
            MemoryArea::new(0x1000, 0x2000, MemoryAreaType::Available),
            MemoryArea::new(0x2000, 0x3000, MemoryAreaType::Available),
        ]);
        dbg!(mmap);
    }

    #[test]
    fn efi_construct_and_parse() {
        let descs = [
            EFIMemoryDesc {
                ty: EFIMemoryAreaType::CONVENTIONAL,
                phys_start: 0x1000,
                virt_start: 0x1000,
                page_count: 1,
                att: Default::default(),
            },
            EFIMemoryDesc {
                ty: EFIMemoryAreaType::LOADER_DATA,
                phys_start: 0x2000,
                virt_start: 0x2000,
                page_count: 3,
                att: Default::default(),
            },
        ];
        let efi_mmap_tag = EFIMemoryMapTag::new_from_descs(&descs);

        let mut iter = efi_mmap_tag.memory_areas();

        assert_eq!(iter.next(), Some(&descs[0]));
        assert_eq!(iter.next(), Some(&descs[1]));

        assert_eq!(iter.next(), None);
    }

    /// Tests the EFI memory map parsing using a real world efi memory map.
    /// This is taken from the uefi-rs repository. See
    /// <https://github.com/rust-osdev/uefi-rs/pull/1175> for more info.
    #[test]
    fn efi_test_real_data() {
        const DESC_SIZE: u32 = 48;
        const DESC_VERSION: u32 = 1;
        /// Sample with 10 entries of a real UEFI memory map extracted from our
        /// UEFI test runner.
        const MMAP_RAW: [u64; 60] = [
            3, 0, 0, 1, 15, 0, 7, 4096, 0, 134, 15, 0, 4, 552960, 0, 1, 15, 0, 7, 557056, 0, 24,
            15, 0, 7, 1048576, 0, 1792, 15, 0, 10, 8388608, 0, 8, 15, 0, 7, 8421376, 0, 3, 15, 0,
            10, 8433664, 0, 1, 15, 0, 7, 8437760, 0, 4, 15, 0, 10, 8454144, 0, 240, 15, 0,
        ];
        let buf = MMAP_RAW;
        let buf = unsafe {
            core::slice::from_raw_parts(buf.as_ptr().cast::<u8>(), buf.len() * size_of::<u64>())
        };
        let tag = EFIMemoryMapTag::new_from_map(DESC_SIZE, DESC_VERSION, buf);
        let entries = tag.memory_areas().copied().collect::<alloc::vec::Vec<_>>();
        let expected = [
            EFIMemoryDesc {
                ty: EFIMemoryAreaType::BOOT_SERVICES_CODE,
                phys_start: 0x0,
                virt_start: 0x0,
                page_count: 0x1,
                att: EFIMemoryAttribute::UNCACHEABLE
                    | EFIMemoryAttribute::WRITE_COMBINE
                    | EFIMemoryAttribute::WRITE_THROUGH
                    | EFIMemoryAttribute::WRITE_BACK,
            },
            EFIMemoryDesc {
                ty: EFIMemoryAreaType::CONVENTIONAL,
                phys_start: 0x1000,
                virt_start: 0x0,
                page_count: 0x86,
                att: EFIMemoryAttribute::UNCACHEABLE
                    | EFIMemoryAttribute::WRITE_COMBINE
                    | EFIMemoryAttribute::WRITE_THROUGH
                    | EFIMemoryAttribute::WRITE_BACK,
            },
            EFIMemoryDesc {
                ty: EFIMemoryAreaType::BOOT_SERVICES_DATA,
                phys_start: 0x87000,
                virt_start: 0x0,
                page_count: 0x1,
                att: EFIMemoryAttribute::UNCACHEABLE
                    | EFIMemoryAttribute::WRITE_COMBINE
                    | EFIMemoryAttribute::WRITE_THROUGH
                    | EFIMemoryAttribute::WRITE_BACK,
            },
            EFIMemoryDesc {
                ty: EFIMemoryAreaType::CONVENTIONAL,
                phys_start: 0x88000,
                virt_start: 0x0,
                page_count: 0x18,
                att: EFIMemoryAttribute::UNCACHEABLE
                    | EFIMemoryAttribute::WRITE_COMBINE
                    | EFIMemoryAttribute::WRITE_THROUGH
                    | EFIMemoryAttribute::WRITE_BACK,
            },
            EFIMemoryDesc {
                ty: EFIMemoryAreaType::CONVENTIONAL,
                phys_start: 0x100000,
                virt_start: 0x0,
                page_count: 0x700,
                att: EFIMemoryAttribute::UNCACHEABLE
                    | EFIMemoryAttribute::WRITE_COMBINE
                    | EFIMemoryAttribute::WRITE_THROUGH
                    | EFIMemoryAttribute::WRITE_BACK,
            },
            EFIMemoryDesc {
                ty: EFIMemoryAreaType::ACPI_NON_VOLATILE,
                phys_start: 0x800000,
                virt_start: 0x0,
                page_count: 0x8,
                att: EFIMemoryAttribute::UNCACHEABLE
                    | EFIMemoryAttribute::WRITE_COMBINE
                    | EFIMemoryAttribute::WRITE_THROUGH
                    | EFIMemoryAttribute::WRITE_BACK,
            },
            EFIMemoryDesc {
                ty: EFIMemoryAreaType::CONVENTIONAL,
                phys_start: 0x808000,
                virt_start: 0x0,
                page_count: 0x3,
                att: EFIMemoryAttribute::UNCACHEABLE
                    | EFIMemoryAttribute::WRITE_COMBINE
                    | EFIMemoryAttribute::WRITE_THROUGH
                    | EFIMemoryAttribute::WRITE_BACK,
            },
            EFIMemoryDesc {
                ty: EFIMemoryAreaType::ACPI_NON_VOLATILE,
                phys_start: 0x80b000,
                virt_start: 0x0,
                page_count: 0x1,
                att: EFIMemoryAttribute::UNCACHEABLE
                    | EFIMemoryAttribute::WRITE_COMBINE
                    | EFIMemoryAttribute::WRITE_THROUGH
                    | EFIMemoryAttribute::WRITE_BACK,
            },
            EFIMemoryDesc {
                ty: EFIMemoryAreaType::CONVENTIONAL,
                phys_start: 0x80c000,
                virt_start: 0x0,
                page_count: 0x4,
                att: EFIMemoryAttribute::UNCACHEABLE
                    | EFIMemoryAttribute::WRITE_COMBINE
                    | EFIMemoryAttribute::WRITE_THROUGH
                    | EFIMemoryAttribute::WRITE_BACK,
            },
            EFIMemoryDesc {
                ty: EFIMemoryAreaType::ACPI_NON_VOLATILE,
                phys_start: 0x810000,
                virt_start: 0x0,
                page_count: 0xf0,
                att: EFIMemoryAttribute::UNCACHEABLE
                    | EFIMemoryAttribute::WRITE_COMBINE
                    | EFIMemoryAttribute::WRITE_THROUGH
                    | EFIMemoryAttribute::WRITE_BACK,
            },
        ];
        assert_eq!(entries.as_slice(), &expected);
    }
}
