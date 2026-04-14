//! Module for [`ElfSectionsTag`].

use crate::{TagHeader, TagType};
use core::fmt::{Debug, Formatter};
use core::mem;
use elf::endian::NativeEndian;
use elf::section::{SectionHeader, SectionHeaderTable};
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

/// Iterator over the ELF section header table.
pub type ElfSectionIter<'a> = elf::parse::ParsingIterator<'a, NativeEndian, SectionHeader>;

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

    /// Get an iterator over the ELF sections.
    #[must_use]
    pub fn sections(&self) -> ElfSectionIter<'_> {
        SectionHeaderTable::new(NativeEndian, self.class(), &self.sections).into_iter()
    }

    const fn class(&self) -> elf::file::Class {
        use elf::section::{Elf32_Shdr, Elf64_Shdr};
        const SHDR_ELF32_SIZE: usize = size_of::<Elf32_Shdr>();
        const SHDR_ELF64_SIZE: usize = size_of::<Elf64_Shdr>();

        match self.entry_size as usize {
            SHDR_ELF32_SIZE => elf::file::Class::ELF32,
            SHDR_ELF64_SIZE => elf::file::Class::ELF64,
            _ => {
                panic!("Unknown ELF section entry size");
            }
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

impl<'a> From<&'a ElfSectionsTag> for SectionHeaderTable<'a, NativeEndian> {
    fn from(value: &'a ElfSectionsTag) -> Self {
        SectionHeaderTable::new(NativeEndian, value.class(), &value.sections)
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
            .field("sections", &self.sections())
            .finish()
    }
}

/// Extension trait for [SectionHeader] containing getters for rust-native types
pub trait ElfSectionExt {
    /// Get the section type as an `ElfSectionType` enum variant.
    #[must_use]
    fn section_type(&self) -> ElfSectionType;

    /// Get the section's flags.
    #[must_use]
    fn flags(&self) -> ElfSectionFlags;
}

impl ElfSectionExt for SectionHeader {
    fn section_type(&self) -> ElfSectionType {
        match self.sh_type {
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
                log::warn!("Unknown section type {e:x}. Treating as ElfSectionType::Unused");
                ElfSectionType::Unused
            }
        }
    }

    fn flags(&self) -> ElfSectionFlags {
        ElfSectionFlags::from_bits_retain(self.sh_flags)
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
