//! Module for [`ModuleTag`].

use crate::tag::TagHeader;
use crate::{parse_slice_as_string, StringError, TagIter, TagType};
use core::fmt::{Debug, Formatter};
use core::mem;
use multiboot2_common::{MaybeDynSized, Tag};
#[cfg(feature = "builder")]
use {alloc::boxed::Box, multiboot2_common::new_boxed};

/// The module tag can occur multiple times and specifies passed boot modules
/// (blobs in memory). The tag itself doesn't include the blog, but references
/// its location.
#[derive(ptr_meta::Pointee, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C, align(8))]
pub struct ModuleTag {
    header: TagHeader,
    mod_start: u32,
    mod_end: u32,
    /// Null-terminated UTF-8 string
    cmdline: [u8],
}

impl ModuleTag {
    /// Constructs a new tag.
    #[cfg(feature = "builder")]
    #[must_use]
    pub fn new(start: u32, end: u32, cmdline: &str) -> Box<Self> {
        let header = TagHeader::new(Self::ID, 0);
        assert!(end > start, "must have a size");

        let start = start.to_ne_bytes();
        let end = end.to_ne_bytes();
        let cmdline = cmdline.as_bytes();

        if cmdline.ends_with(&[0]) {
            new_boxed(header, &[&start, &end, cmdline])
        } else {
            new_boxed(header, &[&start, &end, cmdline, &[0]])
        }
    }

    /// Reads the command line of the boot module as Rust string slice without
    /// the null-byte.
    /// This is an null-terminated UTF-8 string. If this returns `Err` then perhaps the memory
    /// is invalid or the bootloader doesn't follow the spec.
    ///
    /// For example, this returns `"--test cmdline-option"`.if the GRUB config
    /// contains  `"module2 /some_boot_module --test cmdline-option"`.
    ///
    /// If the function returns `Err` then perhaps the memory is invalid.
    pub fn cmdline(&self) -> Result<&str, StringError> {
        parse_slice_as_string(&self.cmdline)
    }

    /// Start address of the module.
    #[must_use]
    pub const fn start_address(&self) -> u32 {
        self.mod_start
    }

    /// End address of the module
    #[must_use]
    pub const fn end_address(&self) -> u32 {
        self.mod_end
    }

    /// The size of the module/the BLOB in memory.
    #[must_use]
    pub const fn module_size(&self) -> u32 {
        self.mod_end - self.mod_start
    }
}

impl MaybeDynSized for ModuleTag {
    type Header = TagHeader;

    const BASE_SIZE: usize = mem::size_of::<TagHeader>() + 2 * mem::size_of::<u32>();

    fn dst_len(header: &TagHeader) -> usize {
        assert!(header.size as usize >= Self::BASE_SIZE);
        header.size as usize - Self::BASE_SIZE
    }
}

impl Tag for ModuleTag {
    type IDType = TagType;

    const ID: TagType = TagType::Module;
}

impl Debug for ModuleTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ModuleTag")
            .field("type", &self.header.typ)
            .field("size", &self.header.size)
            // Trick to print as hex.
            .field("mod_start", &self.mod_start)
            .field("mod_end", &self.mod_end)
            .field("mod_size", &self.module_size())
            .field("cmdline", &self.cmdline())
            .finish()
    }
}

pub const fn module_iter(iter: TagIter) -> ModuleIter {
    ModuleIter { iter }
}

/// An iterator over all module tags.
#[derive(Clone)]
pub struct ModuleIter<'a> {
    iter: TagIter<'a>,
}

impl<'a> Iterator for ModuleIter<'a> {
    type Item = &'a ModuleTag;

    fn next(&mut self) -> Option<&'a ModuleTag> {
        self.iter
            .find(|tag| tag.header().typ == TagType::Module)
            .map(|tag| tag.cast())
    }
}

impl<'a> Debug for ModuleIter<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut list = f.debug_list();
        self.clone().for_each(|tag| {
            list.entry(&tag);
        });
        list.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GenericInfoTag;
    use core::borrow::Borrow;
    use multiboot2_common::test_utils::AlignedBytes;

    #[rustfmt::skip]
    fn get_bytes() -> AlignedBytes<24> {
        AlignedBytes::new([
            TagType::Module.val() as u8, 0, 0, 0,
            22, 0, 0, 0,
            /* mod start */
            0x00, 0xff, 0, 0,
            /* mod end */
            0xff, 0xff, 0, 0,
            b'h', b'e', b'l', b'l', b'o', b'\0',
            /* padding */
            0, 0,
        ])
    }

    /// Tests to parse a string with a terminating null byte from the tag (as the spec defines).
    #[test]
    fn test_parse_str() {
        let bytes = get_bytes();
        let tag = GenericInfoTag::ref_from_slice(bytes.borrow()).unwrap();
        let tag = tag.cast::<ModuleTag>();
        assert_eq!(tag.header.typ, TagType::Module);
        assert_eq!(tag.cmdline(), Ok("hello"));
    }

    /// Test to generate a tag from a given string.
    #[test]
    #[cfg(feature = "builder")]
    fn test_build_str() {
        let tag = ModuleTag::new(0xff00, 0xffff, "hello");
        let bytes = tag.as_bytes().as_ref();
        let bytes = &bytes[..tag.header.size as usize];
        assert_eq!(bytes, &get_bytes()[..tag.header().size as usize]);
        assert_eq!(tag.cmdline(), Ok("hello"));

        // With terminating null.
        let tag = ModuleTag::new(0xff00, 0xffff, "hello\0");
        let bytes = tag.as_bytes().as_ref();
        let bytes = &bytes[..tag.header.size as usize];
        assert_eq!(bytes, &get_bytes()[..tag.header().size as usize]);
        assert_eq!(tag.cmdline(), Ok("hello"));

        // test also some bigger message
        let tag = ModuleTag::new(0, 1, "AbCdEfGhUjK YEAH");
        assert_eq!(tag.cmdline(), Ok("AbCdEfGhUjK YEAH"));
        let tag = ModuleTag::new(0, 1, "AbCdEfGhUjK YEAH".repeat(42).as_str());
        assert_eq!(tag.cmdline(), Ok("AbCdEfGhUjK YEAH".repeat(42).as_str()));
    }
}
