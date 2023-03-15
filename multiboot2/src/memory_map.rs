//! Module for [`MemoryMapTag`], [`EFIMemoryMapTag`] and [`BasicMemoryInfoTag`]
//! and corresponding helper types.

pub use uefi_raw::table::boot::MemoryDescriptor as EFIMemoryDesc;
pub use uefi_raw::table::boot::MemoryType as EFIMemoryAreaType;

use crate::{Tag, TagTrait, TagType, TagTypeId};
use core::fmt::{Debug, Formatter};
use core::marker::PhantomData;
use core::mem;
#[cfg(feature = "builder")]
use {crate::builder::AsBytes, crate::builder::BoxedDst};

const METADATA_SIZE: usize = mem::size_of::<TagTypeId>() + 3 * mem::size_of::<u32>();

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
#[repr(C)]
pub struct MemoryMapTag {
    typ: TagTypeId,
    size: u32,
    entry_size: u32,
    entry_version: u32,
    areas: [MemoryArea],
}

impl MemoryMapTag {
    #[cfg(feature = "builder")]
    pub fn new(areas: &[MemoryArea]) -> BoxedDst<Self> {
        let entry_size: u32 = mem::size_of::<MemoryArea>().try_into().unwrap();
        let entry_version: u32 = 0;
        let mut bytes = [entry_size.to_le_bytes(), entry_version.to_le_bytes()].concat();
        for area in areas {
            bytes.extend(area.as_bytes());
        }
        BoxedDst::new(bytes.as_slice())
    }

    /// Returns the entry size.
    pub fn entry_size(&self) -> u32 {
        self.entry_size
    }

    /// Returns the entry version.
    pub fn entry_version(&self) -> u32 {
        self.entry_version
    }

    /// Return the slice with all memory areas.
    pub fn memory_areas(&self) -> &[MemoryArea] {
        // If this ever fails, we need to model this differently in this crate.
        assert_eq!(self.entry_size as usize, mem::size_of::<MemoryArea>());
        &self.areas
    }

    /// Return a mutable slice with all memory areas.
    pub fn all_memory_areas_mut(&mut self) -> &mut [MemoryArea] {
        // If this ever fails, we need to model this differently in this crate.
        assert_eq!(self.entry_size as usize, mem::size_of::<MemoryArea>());
        &mut self.areas
    }
}

impl TagTrait for MemoryMapTag {
    const ID: TagType = TagType::Mmap;

    fn dst_size(base_tag: &Tag) -> usize {
        assert!(base_tag.size as usize >= METADATA_SIZE);
        let size = base_tag.size as usize - METADATA_SIZE;
        assert_eq!(size % mem::size_of::<MemoryArea>(), 0);
        size / mem::size_of::<MemoryArea>()
    }
}

/// A memory area entry descriptor.
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
    pub fn start_address(&self) -> u64 {
        self.base_addr
    }

    /// The end address of the memory region.
    pub fn end_address(&self) -> u64 {
        self.base_addr + self.length
    }

    /// The size, in bytes, of the memory region.
    pub fn size(&self) -> u64 {
        self.length
    }

    /// The type of the memory region.
    pub fn typ(&self) -> MemoryAreaTypeId {
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

#[cfg(feature = "builder")]
impl AsBytes for MemoryArea {}

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
        let val: MemoryAreaTypeId = (*other).into();
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
    typ: TagTypeId,
    size: u32,
    memory_lower: u32,
    memory_upper: u32,
}

impl BasicMemoryInfoTag {
    pub fn new(memory_lower: u32, memory_upper: u32) -> Self {
        Self {
            typ: Self::ID.into(),
            size: mem::size_of::<BasicMemoryInfoTag>().try_into().unwrap(),
            memory_lower,
            memory_upper,
        }
    }

    pub fn memory_lower(&self) -> u32 {
        self.memory_lower
    }

    pub fn memory_upper(&self) -> u32 {
        self.memory_upper
    }
}

impl TagTrait for BasicMemoryInfoTag {
    const ID: TagType = TagType::BasicMeminfo;

    fn dst_size(_base_tag: &Tag) {}
}

const EFI_METADATA_SIZE: usize = mem::size_of::<TagTypeId>() + 3 * mem::size_of::<u32>();

#[cfg(feature = "builder")]
impl AsBytes for EFIMemoryDesc {}

/// EFI memory map tag. The [`EFIMemoryDesc`] follows the EFI specification.
#[derive(ptr_meta::Pointee, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct EFIMemoryMapTag {
    typ: TagTypeId,
    size: u32,
    desc_size: u32,
    desc_version: u32,
    descs: [EFIMemoryDesc],
}

impl EFIMemoryMapTag {
    #[cfg(feature = "builder")]
    /// Create a new EFI memory map tag with the given memory descriptors.
    /// Version and size can't be set because you're passing a slice of
    /// EFIMemoryDescs, not the ones you might have gotten from the firmware.
    pub fn new(descs: &[EFIMemoryDesc]) -> BoxedDst<Self> {
        // update this when updating EFIMemoryDesc
        const MEMORY_DESCRIPTOR_VERSION: u32 = 1;
        let mut bytes = [
            (mem::size_of::<EFIMemoryDesc>() as u32).to_le_bytes(),
            MEMORY_DESCRIPTOR_VERSION.to_le_bytes(),
        ]
        .concat();
        for desc in descs {
            bytes.extend(desc.as_bytes());
        }
        BoxedDst::new(bytes.as_slice())
    }

    /// Return an iterator over ALL marked memory areas.
    ///
    /// This differs from `MemoryMapTag` as for UEFI, the OS needs some non-
    /// available memory areas for tables and such.
    pub fn memory_areas(&self) -> EFIMemoryAreaIter {
        let self_ptr = self as *const EFIMemoryMapTag;
        let start_area = (&self.descs[0]) as *const EFIMemoryDesc;
        EFIMemoryAreaIter {
            current_area: start_area as u64,
            // NOTE: `last_area` is only a bound, it doesn't necessarily point exactly to the last element
            last_area: (self_ptr as *const () as u64 + self.size as u64),
            entry_size: self.desc_size,
            phantom: PhantomData,
        }
    }
}

impl TagTrait for EFIMemoryMapTag {
    const ID: TagType = TagType::EfiMmap;

    fn dst_size(base_tag: &Tag) -> usize {
        assert!(base_tag.size as usize >= EFI_METADATA_SIZE);
        let size = base_tag.size as usize - EFI_METADATA_SIZE;
        assert_eq!(size % mem::size_of::<EFIMemoryDesc>(), 0);
        size / mem::size_of::<EFIMemoryDesc>()
    }
}

/// An iterator over ALL EFI memory areas.
#[derive(Clone, Debug)]
pub struct EFIMemoryAreaIter<'a> {
    current_area: u64,
    last_area: u64,
    entry_size: u32,
    phantom: PhantomData<&'a EFIMemoryDesc>,
}

impl<'a> Iterator for EFIMemoryAreaIter<'a> {
    type Item = &'a EFIMemoryDesc;
    fn next(&mut self) -> Option<&'a EFIMemoryDesc> {
        if self.current_area > self.last_area {
            None
        } else {
            let area = unsafe { &*(self.current_area as *const EFIMemoryDesc) };
            self.current_area += self.entry_size as u64;
            Some(area)
        }
    }
}
