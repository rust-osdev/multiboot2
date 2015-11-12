#![feature(no_std)]
#![no_std]

pub use elf_sections::{ElfSectionsTag, ElfSection, ElfSectionIter, ElfSectionType, ElfSectionFlags};

mod elf_sections;

pub unsafe fn load(address: usize) -> &'static Multiboot {
    let multiboot = &*(address as *const Multiboot);
    assert!(multiboot.has_valid_end_tag());
    multiboot
}

#[repr(C)]
pub struct Multiboot {
    pub total_size: u32,
    _reserved: u32,
    first_tag: Tag,
}

impl Multiboot {
    pub fn elf_sections_tag(&self) -> Option<&'static ElfSectionsTag> {
        self.get_tag(9).map(|tag| unsafe{&*(tag as *const Tag as *const ElfSectionsTag)})
    }

    fn has_valid_end_tag(&self) -> bool {
        const END_TAG: Tag = Tag{typ:0, size:8};

        let self_ptr = self as *const _;
        let end_tag_addr = self_ptr as usize + (self.total_size - END_TAG.size) as usize;
        let end_tag = unsafe{&*(end_tag_addr as *const Tag)};

        end_tag.typ == END_TAG.typ && end_tag.size == END_TAG.size
    }

    fn get_tag(&self, typ: u32) -> Option<&'static Tag> {
        self.tags().find(|tag| tag.typ == typ)
    }

    fn tags(&self) -> TagIter {
        TagIter{current: &self.first_tag as *const _}
    }
}

#[repr(C)]
struct Tag {
    typ: u32,
    size: u32,
    // tag specific fields
}

struct TagIter {
    current: *const Tag,
}

impl Iterator for TagIter {
    type Item = &'static Tag;

    fn next(&mut self) -> Option<&'static Tag> {
        match unsafe{&*self.current} {
            &Tag{typ:0, size:8} => None, // end tag
            tag => {
                // go to next tag
                let mut tag_addr = self.current as usize;
                tag_addr += tag.size as usize;
                tag_addr = ((tag_addr-1) & !0x7) + 0x8; //align at 8 byte
                self.current = tag_addr as *const _;

                Some(tag)
            },
        }
    }
}
