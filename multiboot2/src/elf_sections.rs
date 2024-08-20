//! Module for [`ElfSectionsTag`].

use crate::{TagHeader, TagType};
use core::fmt::{Debug, Formatter};
use core::marker::PhantomData;
use core::mem;
use core::str::Utf8Error;
use multiboot2_common::{MaybeDynSized, Tag};
#[cfg(feature = "builder")]
use {alloc::boxed::Box, multiboot2_common::new_boxed};

/// This tag contains the section header table from an ELF binary.
// The sections iterator is provided via the [`ElfSectionsTag::sections`]
// method.
#[derive(ptr_meta::Pointee, PartialEq, Eq)]
#[repr(C, align(8))]
pub struct ElfSectionsTag {
    header: TagHeader,
    number_of_sections: u32,
    entry_size: u32,
    shndx: u32,
    sections: [u8],
}

impl ElfSectionsTag {
    /// Create a new ElfSectionsTag with the given data.
    #[cfg(feature = "builder")]
    #[must_use]
    pub fn new(number_of_sections: u32, entry_size: u32, shndx: u32, sections: &[u8]) -> Box<Self> {
        let header = TagHeader::new(Self::ID, 0);
        let number_of_sections = number_of_sections.to_ne_bytes();
        let entry_size = entry_size.to_ne_bytes();
        let shndx = shndx.to_ne_bytes();
        new_boxed(
            header,
            &[&number_of_sections, &entry_size, &shndx, sections],
        )
    }

    /// Get an iterator of loaded ELF sections.
    #[must_use]
    pub(crate) const fn sections_iter(&self) -> ElfSectionIter {
        let string_section_offset = (self.shndx * self.entry_size) as isize;
        let string_section_ptr =
            unsafe { self.sections.as_ptr().offset(string_section_offset) as *const _ };
        ElfSectionIter {
            current_section: self.sections.as_ptr(),
            remaining_sections: self.number_of_sections,
            entry_size: self.entry_size,
            string_section: string_section_ptr,
            _phantom_data: PhantomData,
        }
    }

    /// Returns the amount of sections.
    #[must_use]
    pub const fn number_of_sections(&self) -> u32 {
        self.number_of_sections
    }

    /// Returns the size of each entry.
    #[must_use]
    pub const fn entry_size(&self) -> u32 {
        self.entry_size
    }

    /// Returns the index of the section header string table.
    #[must_use]
    pub const fn shndx(&self) -> u32 {
        self.shndx
    }
}

impl MaybeDynSized for ElfSectionsTag {
    type Header = TagHeader;

    const BASE_SIZE: usize = mem::size_of::<TagHeader>() + 3 * mem::size_of::<u32>();

    fn dst_len(header: &TagHeader) -> usize {
        assert!(header.size as usize >= Self::BASE_SIZE);
        header.size as usize - Self::BASE_SIZE
    }
}

impl Tag for ElfSectionsTag {
    type IDType = TagType;

    const ID: TagType = TagType::ElfSections;
}

impl Debug for ElfSectionsTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ElfSectionsTag")
            .field("typ", &self.header.typ)
            .field("size", &self.header.size)
            .field("number_of_sections", &self.number_of_sections)
            .field("entry_size", &self.entry_size)
            .field("shndx", &self.shndx)
            .field("sections", &self.sections_iter())
            .finish()
    }
}

/// An iterator over some ELF sections.
#[derive(Clone)]
pub struct ElfSectionIter<'a> {
    current_section: *const u8,
    remaining_sections: u32,
    entry_size: u32,
    string_section: *const u8,
    _phantom_data: PhantomData<&'a ()>,
}

impl<'a> Iterator for ElfSectionIter<'a> {
    type Item = ElfSection<'a>;

    fn next(&mut self) -> Option<ElfSection<'a>> {
        while self.remaining_sections != 0 {
            let section = ElfSection {
                inner: self.current_section,
                string_section: self.string_section,
                entry_size: self.entry_size,
                _phantom: PhantomData,
            };

            self.current_section = unsafe { self.current_section.offset(self.entry_size as isize) };
            self.remaining_sections -= 1;

            if section.section_type() != ElfSectionType::Unused {
                return Some(section);
            }
        }
        None
    }
}

impl<'a> Debug for ElfSectionIter<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut debug = f.debug_list();
        self.clone().for_each(|ref e| {
            debug.entry(e);
        });
        debug.finish()
    }
}

/// A single generic ELF Section.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ElfSection<'a> {
    inner: *const u8,
    string_section: *const u8,
    entry_size: u32,
    _phantom: PhantomData<&'a ()>,
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
struct ElfSectionInner32 {
    name_index: u32,
    typ: u32,
    flags: u32,
    addr: u32,
    offset: u32,
    size: u32,
    link: u32,
    info: u32,
    addralign: u32,
    entry_size: u32,
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
struct ElfSectionInner64 {
    name_index: u32,
    typ: u32,
    flags: u64,
    addr: u64,
    offset: u64,
    size: u64,
    link: u32,
    info: u32,
    addralign: u64,
    entry_size: u64,
}

impl<'a> ElfSection<'a> {
    /// Get the section type as a `ElfSectionType` enum variant.
    #[must_use]
    pub fn section_type(&self) -> ElfSectionType {
        match self.get().typ() {
            0 => ElfSectionType::Unused,
            1 => ElfSectionType::ProgramSection,
            2 => ElfSectionType::LinkerSymbolTable,
            3 => ElfSectionType::StringTable,
            4 => ElfSectionType::RelaRelocation,
            5 => ElfSectionType::SymbolHashTable,
            6 => ElfSectionType::DynamicLinkingTable,
            7 => ElfSectionType::Note,
            8 => ElfSectionType::Uninitialized,
            9 => ElfSectionType::RelRelocation,
            10 => ElfSectionType::Reserved,
            11 => ElfSectionType::DynamicLoaderSymbolTable,
            0x6000_0000..=0x6FFF_FFFF => ElfSectionType::EnvironmentSpecific,
            0x7000_0000..=0x7FFF_FFFF => ElfSectionType::ProcessorSpecific,
            e => {
                log::warn!(
                    "Unknown section type {:x}. Treating as ElfSectionType::Unused",
                    e
                );
                ElfSectionType::Unused
            }
        }
    }

    /// Get the "raw" section type as a `u32`
    #[must_use]
    pub fn section_type_raw(&self) -> u32 {
        self.get().typ()
    }

    /// Read the name of the section.
    pub fn name(&self) -> Result<&str, Utf8Error> {
        use core::{slice, str};

        let name_ptr = unsafe { self.string_table().offset(self.get().name_index() as isize) };

        // strlen without null byte
        let strlen = {
            let mut len = 0;
            while unsafe { *name_ptr.offset(len) } != 0 {
                len += 1;
            }
            len as usize
        };

        str::from_utf8(unsafe { slice::from_raw_parts(name_ptr, strlen) })
    }

    /// Get the physical start address of the section.
    #[must_use]
    pub fn start_address(&self) -> u64 {
        self.get().addr()
    }

    /// Get the physical end address of the section.
    ///
    /// This is the same as doing `section.start_address() + section.size()`
    #[must_use]
    pub fn end_address(&self) -> u64 {
        self.get().addr() + self.get().size()
    }

    /// Get the section's size in bytes.
    #[must_use]
    pub fn size(&self) -> u64 {
        self.get().size()
    }

    /// Get the section's address alignment constraints.
    ///
    /// That is, the value of `start_address` must be congruent to 0,
    /// modulo the value of `addrlign`. Currently, only 0 and positive
    /// integral powers of two are allowed. Values 0 and 1 mean the section has no
    /// alignment constraints.
    #[must_use]
    pub fn addralign(&self) -> u64 {
        self.get().addralign()
    }

    /// Get the section's flags.
    #[must_use]
    pub fn flags(&self) -> ElfSectionFlags {
        ElfSectionFlags::from_bits_truncate(self.get().flags())
    }

    /// Check if the `ALLOCATED` flag is set in the section flags.
    #[must_use]
    pub fn is_allocated(&self) -> bool {
        self.flags().contains(ElfSectionFlags::ALLOCATED)
    }

    fn get(&self) -> &dyn ElfSectionInner {
        match self.entry_size {
            40 => unsafe { &*(self.inner as *const ElfSectionInner32) },
            64 => unsafe { &*(self.inner as *const ElfSectionInner64) },
            s => panic!("Unexpected entry size: {}", s),
        }
    }

    unsafe fn string_table(&self) -> *const u8 {
        let addr = match self.entry_size {
            40 => (*(self.string_section as *const ElfSectionInner32)).addr as usize,
            64 => (*(self.string_section as *const ElfSectionInner64)).addr as usize,
            s => panic!("Unexpected entry size: {}", s),
        };
        addr as *const _
    }
}

trait ElfSectionInner {
    fn name_index(&self) -> u32;

    fn typ(&self) -> u32;

    fn flags(&self) -> u64;

    fn addr(&self) -> u64;

    fn size(&self) -> u64;

    fn addralign(&self) -> u64;
}

impl ElfSectionInner for ElfSectionInner32 {
    fn name_index(&self) -> u32 {
        self.name_index
    }

    fn typ(&self) -> u32 {
        self.typ
    }

    fn flags(&self) -> u64 {
        self.flags.into()
    }

    fn addr(&self) -> u64 {
        self.addr.into()
    }

    fn size(&self) -> u64 {
        self.size.into()
    }

    fn addralign(&self) -> u64 {
        self.addralign.into()
    }
}

impl ElfSectionInner for ElfSectionInner64 {
    fn name_index(&self) -> u32 {
        self.name_index
    }

    fn typ(&self) -> u32 {
        self.typ
    }

    fn flags(&self) -> u64 {
        self.flags
    }

    fn addr(&self) -> u64 {
        self.addr
    }

    fn size(&self) -> u64 {
        self.size
    }

    fn addralign(&self) -> u64 {
        self.addralign
    }
}

/// An enum abstraction over raw ELF section types.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u32)]
pub enum ElfSectionType {
    /// This value marks the section header as inactive; it does not have an
    /// associated section. Other members of the section header have undefined
    /// values.
    Unused = 0,

    /// The section holds information defined by the program, whose format and
    /// meaning are determined solely by the program.
    ProgramSection = 1,

    /// This section holds a linker symbol table.
    LinkerSymbolTable = 2,

    /// The section holds a string table.
    StringTable = 3,

    /// The section holds relocation entries with explicit addends, such as type
    /// Elf32_Rela for the 32-bit class of object files. An object file may have
    /// multiple relocation sections.
    RelaRelocation = 4,

    /// The section holds a symbol hash table.
    SymbolHashTable = 5,

    /// The section holds dynamic linking tables.
    DynamicLinkingTable = 6,

    /// This section holds information that marks the file in some way.
    Note = 7,

    /// A section of this type occupies no space in the file but otherwise resembles
    /// `ProgramSection`. Although this section contains no bytes, the
    /// sh_offset member contains the conceptual file offset.
    Uninitialized = 8,

    /// The section holds relocation entries without explicit addends, such as type
    /// Elf32_Rel for the 32-bit class of object files. An object file may have
    /// multiple relocation sections.
    RelRelocation = 9,

    /// This section type is reserved but has unspecified semantics.
    Reserved = 10,

    /// This section holds a dynamic loader symbol table.
    DynamicLoaderSymbolTable = 11,

    /// Values in this inclusive range (`[0x6000_0000, 0x6FFF_FFFF)`) are
    /// reserved for environment-specific semantics.
    EnvironmentSpecific = 0x6000_0000,

    /// Values in this inclusive range (`[0x7000_0000, 0x7FFF_FFFF)`) are
    /// reserved for processor-specific semantics.
    ProcessorSpecific = 0x7000_0000,
}

bitflags! {
    /// ELF Section bitflags.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    #[repr(transparent)]
    pub struct ElfSectionFlags: u64 {
        /// The section contains data that should be writable during program execution.
        const WRITABLE = 0x1;

        /// The section occupies memory during the process execution.
        const ALLOCATED = 0x2;

        /// The section contains executable machine instructions.
        const EXECUTABLE = 0x4;
        // plus environment-specific use at 0x0F000000
        // plus processor-specific use at 0xF0000000
    }
}
