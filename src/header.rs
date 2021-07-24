use core::cmp::Ordering;
use core::fmt::{Debug, Formatter};
use core::marker::PhantomData;

/// Magic number that a multiboot2-compliant boot loader will store in `eax` register
/// right before handoff to the payload (the kernel). This value can be used to check,
/// that the kernel was indeed booted via multiboot2.
///
/// Caution: You might need some assembly code (e.g. GAS or NASM) first, which
/// moves `eax` to another register, like `edi`. Otherwise it probably happens,
/// that the Rust compiler output changes `eax` before you can access it.
pub const MB2_MAGIC: u32 = 0x36d76289;

/// Possible Types of a [`Tag`]. The names and values are taken from the example C code
/// at the bottom of the Multiboot2 specification.
#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum TagType {
    End = 0,
    Cmdline = 1,
    BootLoaderName = 2,
    Module = 3,
    BasicMeminfo = 4,
    Bootdev = 5,
    Mmap = 6,
    Vbe = 7,
    Framebuffer = 8,
    ElfSections = 9,
    Apm = 10,
    Efi32 = 11,
    Efi64 = 12,
    Smbios = 13,
    /// Also called "AcpiOld" in other multiboot2 implementations.
    AcpiV1 = 14,
    /// Refers to version 2 and later of Acpi.
    /// Also called "AcpiNew" in other multiboot2 implementations.
    AcpiV2 = 15,
    Network = 16,
    EfiMmap = 17,
    EfiBs = 18,
    Efi32Ih = 19,
    Efi64Ih = 20,
    LoadBaseAddr = 21,
}

// each compare/equal direction must be implemented manually
impl PartialEq<u32> for TagType {
    fn eq(&self, other: &u32) -> bool {
        *self as u32 == *other
    }
}

// each compare/equal direction must be implemented manually
impl PartialEq<TagType> for u32 {
    fn eq(&self, other: &TagType) -> bool {
        *self == *other as u32
    }
}

impl PartialEq<TagType> for TagType {
    fn eq(&self, other: &TagType) -> bool {
        *self as u32 == *other as u32
    }
}

impl PartialOrd<u32> for TagType {
    fn partial_cmp(&self, other: &u32) -> Option<Ordering> {
        let num = *self as u32;
        num.partial_cmp(other)
    }
}

/// All tags that could passed via the Multiboot2 information structure to a payload/program/kernel.
/// Better not confuse this with the Multiboot2 header tags. They are something different.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Tag {
    // u32 value
    pub typ: TagType,
    pub size: u32,
    // tag specific fields
}

impl Debug for Tag {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Tag")
            .field("typ", &self.typ)
            .field("typ (numeric)", &(self.typ as u32))
            .field("size", &(self.size))
            .finish()
    }
}

#[derive(Clone, Debug)]
pub struct TagIter<'a> {
    pub current: *const Tag,
    phantom: PhantomData<&'a Tag>,
}

impl<'a> TagIter<'a> {
    pub fn new(first: *const Tag) -> Self {
        TagIter {
            current: first,
            phantom: PhantomData,
        }
    }
}

impl<'a> Iterator for TagIter<'a> {
    type Item = &'a Tag;

    fn next(&mut self) -> Option<&'a Tag> {
        match unsafe { &*self.current } {
            &Tag {
                typ: TagType::End,
                size: 8,
            } => None, // end tag
            tag => {
                // go to next tag
                let mut tag_addr = self.current as usize;
                tag_addr += ((tag.size + 7) & !7) as usize; //align at 8 byte
                self.current = tag_addr as *const _;

                Some(tag)
            }
        }
    }
}
