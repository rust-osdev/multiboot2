use core::marker::PhantomData;

/// This Tag provides an initial host memory map.
///
/// The map provided is guaranteed to list all standard RAM that should be
/// available for normal use. This type however includes the regions occupied
/// by kernel, mbi, segments and modules. Kernel must take care not to
/// overwrite these regions.
///
/// This tag may not be provided by some boot loaders on EFI platforms if EFI
/// boot services are enabled and available for the loaded image
/// (EFI boot services not terminated tag exists in Multiboot2 information structure).   
#[derive(Debug)]
#[repr(C)]
pub struct MemoryMapTag {
    typ: u32,
    size: u32,
    entry_size: u32,
    entry_version: u32,
    first_area: MemoryArea,
}

impl MemoryMapTag {
    /// Return an iterator over all AVAILABLE marked memory areas.
    pub fn memory_areas(&self) -> MemoryAreaIter {
        let self_ptr = self as *const MemoryMapTag;
        let start_area = (&self.first_area) as *const MemoryArea;
        MemoryAreaIter {
            current_area: start_area as u64,
            last_area: (self_ptr as u64 + (self.size - self.entry_size) as u64),
            entry_size: self.entry_size,
            phantom: PhantomData,
        }
    }
}

/// A memory area entry descriptor.
#[derive(Debug)]
#[repr(C)]
pub struct MemoryArea {
    base_addr: u64,
    length: u64,
    typ: u32,
    _reserved: u32,
}

impl MemoryArea {
    /// The start address of the memory region.
    pub fn start_address(&self) -> u64 {
        self.base_addr
    }

    /// The end address of the memory region.
    pub fn end_address(&self) -> u64 {
        (self.base_addr + self.length)
    }

    /// The size, in bytes, of the memory region.
    pub fn size(&self) -> u64 {
        self.length
    }

    /// The type of the memory region.
    pub fn typ(&self) -> MemoryAreaType {
        match self.typ {
            1 => MemoryAreaType::Available,
            3 => MemoryAreaType::AcpiAvailable,
            4 => MemoryAreaType::ReservedHibernate,
            5 => MemoryAreaType::Defective,
            _ => MemoryAreaType::Reserved,
        }
    }
}

/// An enum of possible reported region types.
#[derive(Debug, PartialEq, Eq)]
pub enum MemoryAreaType {
    /// A reserved area that must not be used.
    Reserved,

    /// Available memory free to be used by the OS.
    Available,

    /// Usable memory holding ACPI information.
    AcpiAvailable,

    /// Reserved memory which needs to be preserved on hibernation.
    ReservedHibernate,

    /// Memory which is occupied by defective RAM modules.
    Defective,
}

/// An area over Available memory areas.
#[derive(Clone, Debug)]
pub struct MemoryAreaIter<'a> {
    current_area: u64,
    last_area: u64,
    entry_size: u32,
    phantom: PhantomData<&'a MemoryArea>,
}

impl<'a> Iterator for MemoryAreaIter<'a> {
    type Item = &'a MemoryArea;
    fn next(&mut self) -> Option<&'a MemoryArea> {
        if self.current_area > self.last_area {
            None
        } else {
            let area = unsafe{&*(self.current_area as *const MemoryArea)};
            self.current_area = self.current_area + (self.entry_size as u64);
            if area.typ == 1 {
                Some(area)
            } else {self.next()}
        }
    }
}
