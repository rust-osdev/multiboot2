//! Module for [`ModuleTag`].

use crate::tag::StringError;
use crate::{Tag, TagIter, TagTrait, TagType, TagTypeId};
use core::fmt::{Debug, Formatter};
use core::mem::size_of;
#[cfg(feature = "builder")]
use {crate::builder::BoxedDst, alloc::vec::Vec};

const METADATA_SIZE: usize = size_of::<TagTypeId>() + 3 * size_of::<u32>();

/// The module tag can occur multiple times and specifies passed boot modules
/// (blobs in memory). The tag itself doesn't include the blog, but references
/// its location.
#[derive(ptr_meta::Pointee, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct ModuleTag {
    typ: TagTypeId,
    size: u32,
    mod_start: u32,
    mod_end: u32,
    /// Null-terminated UTF-8 string
    cmdline: [u8],
}

impl ModuleTag {
    #[cfg(feature = "builder")]
    pub fn new(start: u32, end: u32, cmdline: &str) -> BoxedDst<Self> {
        assert!(end > start, "must have a size");

        let mut cmdline_bytes: Vec<_> = cmdline.bytes().collect();
        if !cmdline_bytes.ends_with(&[0]) {
            // terminating null-byte
            cmdline_bytes.push(0);
        }
        let start_bytes = start.to_le_bytes();
        let end_bytes = end.to_le_bytes();
        let mut content_bytes = [start_bytes, end_bytes].concat();
        content_bytes.extend_from_slice(&cmdline_bytes);
        BoxedDst::new(&content_bytes)
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
        Tag::parse_slice_as_string(&self.cmdline)
    }

    /// Start address of the module.
    pub fn start_address(&self) -> u32 {
        self.mod_start
    }

    /// End address of the module
    pub fn end_address(&self) -> u32 {
        self.mod_end
    }

    /// The size of the module/the BLOB in memory.
    pub fn module_size(&self) -> u32 {
        self.mod_end - self.mod_start
    }
}

impl TagTrait for ModuleTag {
    const ID: TagType = TagType::Module;

    fn dst_size(base_tag: &Tag) -> usize {
        assert!(base_tag.size as usize >= METADATA_SIZE);
        base_tag.size as usize - METADATA_SIZE
    }
}

impl Debug for ModuleTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ModuleTag")
            .field("type", &{ self.typ })
            .field("size", &{ self.size })
            // Trick to print as hex.
            .field("mod_start", &self.mod_start)
            .field("mod_end", &self.mod_end)
            .field("mod_size", &self.module_size())
            .field("cmdline", &self.cmdline())
            .finish()
    }
}

pub fn module_iter(iter: TagIter) -> ModuleIter {
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
            .find(|tag| tag.typ == TagType::Module)
            .map(|tag| tag.cast_tag())
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
    use crate::{ModuleTag, Tag, TagTrait, TagType};

    const MSG: &str = "hello";

    /// Returns the tag structure in bytes in little endian format.
    fn get_bytes() -> std::vec::Vec<u8> {
        // size is: 4 bytes for tag + 4 bytes for size + length of null-terminated string
        //          4 bytes mod_start + 4 bytes mod_end
        let size = (4 + 4 + 4 + 4 + MSG.as_bytes().len() + 1) as u32;
        [
            &((TagType::Module.val()).to_le_bytes()),
            &size.to_le_bytes(),
            &0_u32.to_le_bytes(),
            &1_u32.to_le_bytes(),
            MSG.as_bytes(),
            // Null Byte
            &[0],
        ]
        .iter()
        .flat_map(|bytes| bytes.iter())
        .copied()
        .collect()
    }

    /// Tests to parse a string with a terminating null byte from the tag (as the spec defines).
    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_parse_str() {
        let tag = get_bytes();
        let tag = unsafe { &*tag.as_ptr().cast::<Tag>() };
        let tag = tag.cast_tag::<ModuleTag>();
        assert_eq!({ tag.typ }, TagType::Module);
        assert_eq!(tag.cmdline().expect("must be valid UTF-8"), MSG);
    }

    /// Test to generate a tag from a given string.
    #[test]
    #[cfg(feature = "builder")]
    fn test_build_str() {
        let tag = ModuleTag::new(0, 1, MSG);
        let bytes = tag.as_bytes();
        assert_eq!(bytes, get_bytes());
        assert_eq!(tag.cmdline(), Ok(MSG));

        // test also some bigger message
        let tag = ModuleTag::new(0, 1, "AbCdEfGhUjK YEAH");
        assert_eq!(tag.cmdline(), Ok("AbCdEfGhUjK YEAH"));
        let tag = ModuleTag::new(0, 1, "AbCdEfGhUjK YEAH".repeat(42).as_str());
        assert_eq!(tag.cmdline(), Ok("AbCdEfGhUjK YEAH".repeat(42).as_str()));
    }
}
