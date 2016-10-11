
#[derive(Debug)]
#[repr(packed)] // repr(C) would add unwanted padding before first_section
pub struct ElfSectionsTag {
    typ: u32,
    size: u32,
    pub number_of_sections: u32,
    entry_size: u32,
    shndx: u32, // string table
    first_section: ElfSection,
}

impl ElfSectionsTag {
    pub fn sections(&'static self) -> ElfSectionIter {
        ElfSectionIter {
            current_section: &self.first_section,
            remaining_sections: self.number_of_sections - 1,
            entry_size: self.entry_size,
        }
    }

    pub fn string_table(&self) -> &'static StringTable {
        unsafe {
            let string_table_ptr =
                (&self.first_section as *const ElfSection).offset(self.shndx as isize);
            &*((*string_table_ptr).addr as *const StringTable)
        }
    }
}

pub struct StringTable(u8);

impl StringTable {
    pub fn section_name(&self, section: &ElfSection) -> &'static str {
        use core::{str, slice};

        let name_ptr = unsafe {
            (&self.0 as *const u8).offset(section.name_index as isize)
        };
        let strlen = {
            let mut len = 0;
            while unsafe { *name_ptr.offset(len) } != 0 {
                len += 1;
            }
            len as usize
        };

        str::from_utf8( unsafe {
            slice::from_raw_parts(name_ptr, strlen)
        }).unwrap()
    }
}

#[derive(Clone)]
pub struct ElfSectionIter {
    current_section: &'static ElfSection,
    remaining_sections: u32,
    entry_size: u32,
}

impl Iterator for ElfSectionIter {
    type Item = &'static ElfSection;
    fn next(&mut self) -> Option<&'static ElfSection> {
        if self.remaining_sections == 0 {
            None
        } else {
            let section = self.current_section;
            let next_section_addr = (self.current_section as *const _ as u64) + self.entry_size as u64;
            self.current_section = unsafe{ &*(next_section_addr as *const ElfSection) };
            self.remaining_sections -= 1;
            if section.typ == ElfSectionType::Unused as u32 {
                self.next()
            } else {
                Some(section)
            }
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct ElfSection {
    name_index: u32,
    typ: u32,
    pub flags: u64,
    pub addr: u64,
    offset: u64,
    pub size: u64,
    link: u32,
    info: u32,
    addralign: u64,
    entry_size: u64,
}

impl ElfSection {
    pub fn start_address(&self) -> usize {
        self.addr as usize
    }

    pub fn end_address(&self) -> usize {
        (self.addr + self.size) as usize
    }

    pub fn flags(&self) -> ElfSectionFlags {
        ElfSectionFlags::from_bits_truncate(self.flags)
    }

    pub fn is_allocated(&self) -> bool {
        self.flags().contains(ELF_SECTION_ALLOCATED)
    }
}

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
    // plus environment-specific use from 0x60000000 to 0x6FFFFFFF
    // plus processor-specific use from 0x70000000 to 0x7FFFFFFF
}

bitflags! {
    flags ElfSectionFlags: u64 {
        const ELF_SECTION_WRITABLE = 0x1,
        const ELF_SECTION_ALLOCATED = 0x2,
        const ELF_SECTION_EXECUTABLE = 0x4,
        // plus environment-specific use at 0x0F000000
        // plus processor-specific use at 0xF0000000
    }
}
