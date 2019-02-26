use header::Tag;

#[derive(Debug)]
pub struct ElfSectionsTag {
    inner: *const ElfSectionsTagInner,
    offset: usize,
}

pub unsafe fn elf_sections_tag(tag: &Tag, offset: usize) -> ElfSectionsTag {
    assert_eq!(9, tag.typ);
    let es = ElfSectionsTag {
        inner: (tag as *const Tag).offset(1) as *const ElfSectionsTagInner,
        offset: offset,
    };
    assert!((es.get().entry_size * es.get().shndx) <= tag.size);
    es
}

#[derive(Clone, Copy, Debug)]
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
            remaining_sections: self.get().number_of_sections,
            entry_size: self.get().entry_size,
            string_section: string_section_ptr,
            offset: self.offset,
        }
    }

    fn first_section(&self) -> *const u8 {
        (unsafe { self.inner.offset(1) }) as *const _
    }

    fn get(&self) -> &ElfSectionsTagInner {
        unsafe { &*self.inner }
    }
}

#[derive(Clone, Debug)]
pub struct ElfSectionIter {
    current_section: *const u8,
    remaining_sections: u32,
    entry_size: u32,
    string_section: *const u8,
    offset: usize,
}

impl Iterator for ElfSectionIter {
    type Item = ElfSection;

    fn next(&mut self) -> Option<ElfSection> {
        while self.remaining_sections != 0 {
            let section = ElfSection {
                inner: self.current_section,
                string_section: self.string_section,
                entry_size: self.entry_size,
                offset: self.offset,
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

#[derive(Debug)]
pub struct ElfSection {
    inner: *const u8,
    string_section: *const u8,
    entry_size: u32,
    offset: usize,
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

impl ElfSection {
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
            0x6000_0000...0x6FFF_FFFF => ElfSectionType::EnvironmentSpecific,
            0x7000_0000...0x7FFF_FFFF => ElfSectionType::ProcessorSpecific,
            _ => panic!(),
        }
    }

    pub fn section_type_raw(&self) -> u32 {
        self.get().typ()
    }

    pub fn name(&self) -> &str {
        use core::{str, slice};

        let name_ptr = unsafe {
            self.string_table().offset(self.get().name_index() as isize)
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

    pub fn start_address(&self) -> u64 {
        self.get().addr()
    }

    pub fn end_address(&self) -> u64 {
        self.get().addr() + self.get().size()
    }

    pub fn size(&self) -> u64 {
        self.get().size()
    }

    pub fn addralign(&self) -> u64 {
        self.get().addralign()
    }

    pub fn flags(&self) -> ElfSectionFlags {
        ElfSectionFlags::from_bits_truncate(self.get().flags())
    }

    pub fn is_allocated(&self) -> bool {
        self.flags().contains(ElfSectionFlags::ALLOCATED)
    }

    fn get(&self) -> &ElfSectionInner {
        match self.entry_size {
            40 => unsafe { &*(self.inner as *const ElfSectionInner32) },
            64 => unsafe { &*(self.inner as *const ElfSectionInner64) },
            _ => panic!(),
        }
    }

    unsafe fn string_table(&self) -> *const u8 {
        let addr = match self.entry_size {
            40 => (*(self.string_section as *const ElfSectionInner32)).addr as usize,
            64 => (*(self.string_section as *const ElfSectionInner64)).addr as usize,
            _ => panic!(),
        };
        (addr + self.offset) as *const _
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
        self.addralign.into()
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

bitflags! {
    pub struct ElfSectionFlags: u64 {
        const WRITABLE = 0x1;
        const ALLOCATED = 0x2;
        const EXECUTABLE = 0x4;
        // plus environment-specific use at 0x0F000000
        // plus processor-specific use at 0xF0000000
    }
}
