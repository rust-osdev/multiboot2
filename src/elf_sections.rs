use header::Tag;

#[derive(Debug)]
pub struct ElfSectionsTag {
    inner: *const ElfSectionsTagInner,
}

pub fn elf_sections_tag(tag: &Tag) -> ElfSectionsTag {
    assert_eq!(9, tag.typ);
    let es = ElfSectionsTag {
        inner: unsafe { (tag as *const _).offset(1) } as *const _,
    };
    assert!((es.get().entry_size * es.get().shndx) <= tag.size);
    es
}

#[derive(Debug)]
#[repr(C, packed)] // only repr(C) would add unwanted padding at the end
struct ElfSectionsTagInner {
    number_of_sections: u32,
    entry_size: u32,
    shndx: u32, // string table
}

impl ElfSectionsTag {
    pub fn sections(&self) -> ElfSectionIter {
        let string_section_offset = (self.get().shndx * self.get().entry_size) as isize;
        let string_section_ptr = unsafe {
            self.first_section().offset(string_section_offset) as *const _
        };
        ElfSectionIter {
            current_section: self.first_section(),
            remaining_sections: self.get().number_of_sections - 1,
            entry_size: self.get().entry_size,
            string_section: string_section_ptr,
        }
    }

    fn first_section(&self) -> *const u8 {
        (unsafe { self.inner.offset(1) }) as *const _
    }

    fn get(&self) -> &ElfSectionsTagInner {
        unsafe { &*self.inner }
    }
}

#[derive(Clone)]
pub struct ElfSectionIter {
    current_section: *const u8,
    remaining_sections: u32,
    entry_size: u32,
    string_section: *const ElfSectionInner,
}

impl Iterator for ElfSectionIter {
    type Item = ElfSection;
    fn next(&mut self) -> Option<ElfSection> {
        if self.remaining_sections == 0 {
            return None;
        }

        loop {
            let section = ElfSection {
                inner: self.current_section as *const ElfSectionInner,
                string_section: self.string_section,
            };

            self.current_section = unsafe { self.current_section.offset(self.entry_size as isize) };
            self.remaining_sections -= 1;

            if section.section_type() != ElfSectionType::Unused {
                return Some(section);
            }
        }
    }
}

pub struct ElfSection {
    inner: *const ElfSectionInner,
    string_section: *const ElfSectionInner,
}

#[cfg(feature = "elf32")]
#[derive(Debug)]
#[repr(C)]
struct ElfSectionInner {
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

#[cfg(not(feature = "elf32"))]
#[derive(Debug)]
#[repr(C)]
struct ElfSectionInner {
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

impl ElfSection {
    pub fn section_type(&self) -> ElfSectionType {
        match self.get().typ {
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
            0x6000_0000...0x6FFF_FFFF => ElfSectionType::EnvironmentSpecific,
            0x7000_0000...0x7FFF_FFFF => ElfSectionType::ProcessorSpecific,
            _ => panic!(),
        }
    }

    pub fn section_type_raw(&self) -> u32 {
        self.get().typ
    }

    pub fn name(&self) -> &str {
        use core::{str, slice};

        let name_ptr = unsafe {
            self.string_table().offset(self.get().name_index as isize)
        };
        let strlen = {
            let mut len = 0;
            while unsafe { *name_ptr.offset(len) } != 0 {
                len += 1;
            }
            len as usize
        };

        str::from_utf8(unsafe { slice::from_raw_parts(name_ptr, strlen) }).unwrap()
    }

    pub fn start_address(&self) -> usize {
        self.get().addr as usize
    }

    pub fn end_address(&self) -> usize {
        (self.get().addr + self.get().size) as usize
    }

    pub fn size(&self) -> usize {
        self.get().size as usize
    }

    pub fn flags(&self) -> ElfSectionFlags {
        ElfSectionFlags::from_bits_truncate(self.get().flags)
    }

    pub fn is_allocated(&self) -> bool {
        self.flags().contains(ELF_SECTION_ALLOCATED)
    }

    fn get(&self) -> &ElfSectionInner {
        unsafe { &*self.inner }
    }

    unsafe fn string_table(&self) -> *const u8 {
        (*self.string_section).addr as *const _
    }
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
#[repr(u32)]
pub enum ElfSectionType {
    Unused = 0,
    ProgramSection = 1,
    LinkerSymbolTable = 2,
    StringTable = 3,
    RelaRelocation = 4,
    SymbolHashTable = 5,
    DynamicLinkingTable = 6,
    Note = 7,
    Uninitialized = 8,
    RelRelocation = 9,
    Reserved = 10,
    DynamicLoaderSymbolTable = 11,
    EnvironmentSpecific = 0x6000_0000,
    ProcessorSpecific = 0x7000_0000,
}

#[cfg(feature = "elf32")]
type ElfSectionFlagsType = u32;

#[cfg(not(feature = "elf32"))]
type ElfSectionFlagsType = u64;

bitflags! {
    flags ElfSectionFlags: ElfSectionFlagsType {
        const ELF_SECTION_WRITABLE = 0x1,
        const ELF_SECTION_ALLOCATED = 0x2,
        const ELF_SECTION_EXECUTABLE = 0x4,
        // plus environment-specific use at 0x0F000000
        // plus processor-specific use at 0xF0000000
    }
}
