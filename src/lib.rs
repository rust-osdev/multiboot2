#![feature(no_std)]
#![no_std]

pub unsafe fn load(address: usize) -> &'static Multiboot {
    let multiboot = &*(address as *const Multiboot);
    assert!(multiboot.has_valid_end_tag());
    multiboot
}

#[repr(C)]
pub struct Multiboot {
    pub total_size: u32,
    _reserved: u32,
    // tags
}

impl Multiboot {
    fn has_valid_end_tag(&self) -> bool {
        const END_TAG: Tag = Tag{typ:0, size:8};

        let self_ptr = self as *const _;
        let end_tag_addr = self_ptr as usize + (self.total_size - END_TAG.size) as usize;
        let end_tag = unsafe{&*(end_tag_addr as *const Tag)};

        end_tag.typ == END_TAG.typ && end_tag.size == END_TAG.size
    }
}

#[repr(C)]
struct Tag {
    typ: u32,
    size: u32,
    // tag specific fields
}
