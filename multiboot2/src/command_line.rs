//! Module for [CommandLineTag].

use crate::TagType;
use core::mem;
use core::slice;
use core::str::Utf8Error;

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
    pub fn command_line(&self) -> Result<&str, Utf8Error> {
        let strlen = self.size as usize - mem::size_of::<CommandLineTag>();
        let bytes = unsafe { slice::from_raw_parts((&self.string) as *const u8, strlen) };
        core::str::from_utf8(bytes)
    }
}
