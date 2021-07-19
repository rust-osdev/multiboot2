//! Type definitions for tags inside the "multiboot2 information structure" (also *mbi*).
//! These tags occur in a binary if it is multiboot2-compliant, for example a kernel.
//!
//! The values are taken from the example C code at the end of the official multiboot2 spec.

// the names speak for themselves in most cases
#![allow(missing_docs)]

use core::cmp::Ordering;
use core::fmt::{Debug, Formatter};
use core::marker::PhantomData;

/// Possible types of a [`Tag`]. The names and values are taken from the example C code
/// at the bottom of the Multiboot2 specification.
#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum MbiTagType {
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
impl PartialEq<u32> for MbiTagType {
    fn eq(&self, other: &u32) -> bool {
        *self as u32 == *other
    }
}

// each compare/equal direction must be implemented manually
impl PartialEq<MbiTagType> for u32 {
    fn eq(&self, other: &MbiTagType) -> bool {
        *self == *other as u32
    }
}

impl PartialEq<MbiTagType> for MbiTagType {
    fn eq(&self, other: &MbiTagType) -> bool {
        *self as u32 == *other as u32
    }
}

impl PartialOrd<u32> for MbiTagType {
    fn partial_cmp(&self, other: &u32) -> Option<Ordering> {
        let num = *self as u32;
        Some(if num < *other {
            Ordering::Less
        } else if num == *other {
            Ordering::Equal
        } else {
            Ordering::Greater
        })
    }
}

/// All tags that could passed via the Multiboot2 information structure ("mbi") to a
/// payload/program/kernel. Better not confuse this with the Multiboot2 header tags. They are
/// something different.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct MbiTag {
    // u32 value
    pub typ: MbiTagType,
    pub size: u32,
    // tag specific fields
}

impl Debug for MbiTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Tag")
            .field("typ", &self.typ)
            .field("typ (numeric)", &(self.typ as u32))
            .field("size", &(self.size))
            .finish()
    }
}

#[derive(Clone, Debug)]
pub struct MbiTagIter<'a> {
    pub current: *const MbiTag,
    phantom: PhantomData<&'a MbiTag>,
}

impl<'a> MbiTagIter<'a> {
    pub fn new(first: *const MbiTag) -> Self {
        MbiTagIter {
            current: first,
            phantom: PhantomData,
        }
    }
}

impl<'a> Iterator for MbiTagIter<'a> {
    type Item = &'a MbiTag;

    fn next(&mut self) -> Option<&'a MbiTag> {
        match unsafe { &*self.current } {
            &MbiTag {
                typ: MbiTagType::End,
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
