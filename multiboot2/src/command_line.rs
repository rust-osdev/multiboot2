//! Module for [CommandLineTag].

use crate::TagType;
use core::mem;
use core::slice;
use core::str;

/// This tag contains the command line string.
///
/// The string is a normal C-style UTF-8 zero-terminated string that can be
/// obtained via the `command_line` method.
#[derive(Clone, Copy, Debug)]
#[repr(C, packed)] // only repr(C) would add unwanted padding before first_section
pub struct CommandLineTag {
    typ: TagType,
    size: u32,
    /// Null-terminated UTF-8 string
    string: u8,
}

impl CommandLineTag {
    /// Read the command line string that is being passed to the booting kernel.
    /// This is an null-terminated UTF-8 string. If this returns `Err` then perhaps the memory
    /// is invalid or the bootloader doesn't follow the spec.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// if let Some(tag) = boot_info.command_line_tag() {
    ///     let command_line = tag.command_line();
    ///     assert_eq!("/bootarg", command_line);
    /// }
    /// ```
    pub fn command_line(&self) -> Result<&str, str::Utf8Error> {
        // strlen without null byte
        let strlen = self.size as usize - mem::size_of::<CommandLineTag>();
        let bytes = unsafe { slice::from_raw_parts((&self.string) as *const u8, strlen) };
        str::from_utf8(bytes)
    }
}

#[cfg(test)]
mod tests {
    use crate::TagType;

    const MSG: &str = "hello";

    /// Returns the tag structure in bytes in native endian format.
    fn get_bytes() -> std::vec::Vec<u8> {
        // size is: 4 bytes for tag + 4 bytes for size + length of null-terminated string
        let size = (4 + 4 + MSG.as_bytes().len() + 1) as u32;
        [
            &((TagType::Cmdline as u32).to_ne_bytes()),
            &size.to_ne_bytes(),
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
        let tag = unsafe {
            tag.as_ptr()
                .cast::<super::CommandLineTag>()
                .as_ref()
                .unwrap()
        };
        assert_eq!({ tag.typ }, TagType::Cmdline);
        assert_eq!(tag.command_line().expect("must be valid UTF-8"), MSG);
    }
}
