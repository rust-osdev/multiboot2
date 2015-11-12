
#[derive(Debug)]
#[repr(C)]
pub struct ElfSectionsTag {
    typ: u32,
    size: u32,
    pub number_of_sections: u32,
    entry_size: u32,
    shndx: u32,
    first_section: ElfSection,
}

impl ElfSectionsTag {
    pub fn sections(&self) -> ElfSectionIter {
        let start_section = (&self.first_section) as *const _;
        ElfSectionIter {
            current_section: start_section,
            remaining_sections: self.number_of_sections - 1,
            entry_size: self.entry_size,
        }
    }
}

#[derive(Clone)]
#[allow(raw_pointer_derive)]
pub struct ElfSectionIter {
    current_section: *const ElfSection,
    remaining_sections: u32,
    entry_size: u32,
}

impl Iterator for ElfSectionIter {
    type Item = &'static ElfSection;
    fn next(&mut self) -> Option<&'static ElfSection> {
        if self.remaining_sections == 0 {
            None
        } else {
            let section = unsafe{&*self.current_section};
            self.current_section = ((self.current_section as u32) +
                self.entry_size) as *const ElfSection;
            self.remaining_sections -= 1;
			if section.typ == 0 {
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
    name: u32,
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
