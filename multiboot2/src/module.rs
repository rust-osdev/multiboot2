use crate::tag_type::{Tag, TagIter, TagType};
use crate::TagTypeId;
use core::fmt::{Debug, Formatter};
use core::str::Utf8Error;

/// This tag indicates to the kernel what boot module was loaded along with
/// the kernel image, and where it can be found.
#[derive(Clone, Copy)]
#[repr(C, packed)] // only repr(C) would add unwanted padding near name_byte.
pub struct ModuleTag {
    typ: TagTypeId,
    size: u32,
    mod_start: u32,
    mod_end: u32,
    /// Null-terminated UTF-8 string
    cmdline_str: u8,
}

impl ModuleTag {
    /// Returns the cmdline of the module.
    /// This is an null-terminated UTF-8 string. If this returns `Err` then perhaps the memory
    /// is invalid or the bootloader doesn't follow the spec.
    ///
    /// For example: If the GRUB configuration contains
    /// `module2 /foobar/some_boot_module --test cmdline-option` then this method
    /// will return `--test cmdline-option`.
    pub fn cmdline(&self) -> Result<&str, Utf8Error> {
        use core::{mem, slice, str};
        // strlen without null byte
        let strlen = self.size as usize - mem::size_of::<ModuleTag>();
        let bytes = unsafe { slice::from_raw_parts((&self.cmdline_str) as *const u8, strlen) };
        str::from_utf8(bytes)
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

impl Debug for ModuleTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ModuleTag")
            .field("type", &{ self.typ })
            .field("size (tag)", &{ self.size })
            .field("size (module)", &self.module_size())
            .field("mod_start", &(self.mod_start as *const usize))
            .field("mod_end", &(self.mod_end as *const usize))
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
            .map(|tag| unsafe { &*(tag as *const Tag as *const ModuleTag) })
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
    use crate::TagType;

    const MSG: &str = "hello";

    /// Returns the tag structure in bytes in native endian format.
    fn get_bytes() -> std::vec::Vec<u8> {
        // size is: 4 bytes for tag + 4 bytes for size + length of null-terminated string
        //          4 bytes mod_start + 4 bytes mod_end
        let size = (4 + 4 + 4 + 4 + MSG.as_bytes().len() + 1) as u32;
        [
            &((TagType::Module.val()).to_ne_bytes()),
            &size.to_ne_bytes(),
            &0_u32.to_ne_bytes(),
            &0_u32.to_ne_bytes(),
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
    fn test_parse_str() {
        let tag = get_bytes();
        let tag = unsafe { tag.as_ptr().cast::<super::ModuleTag>().as_ref().unwrap() };
        assert_eq!({ tag.typ }, TagType::Module);
        assert_eq!(tag.cmdline().expect("must be valid UTF-8"), MSG);
    }
}
