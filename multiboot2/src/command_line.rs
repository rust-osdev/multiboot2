//! Module for [`CommandLineTag`].

use crate::tag::TagHeader;
use crate::{parse_slice_as_string, StringError, TagTrait, TagType};
use core::fmt::{Debug, Formatter};
use core::mem;
use core::str;
#[cfg(feature = "builder")]
use {crate::new_boxed, alloc::boxed::Box, alloc::vec::Vec};

const METADATA_SIZE: usize = mem::size_of::<TagHeader>();

/// This tag contains the command line string.
///
/// The string is a normal C-style UTF-8 zero-terminated string that can be
/// obtained via the `command_line` method.
#[derive(ptr_meta::Pointee, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C, align(8))]
pub struct CommandLineTag {
    header: TagHeader,
    /// Null-terminated UTF-8 string
    cmdline: [u8],
}

impl CommandLineTag {
    /// Create a new command line tag from the given string.
    #[cfg(feature = "builder")]
    #[must_use]
    pub fn new(command_line: &str) -> Box<Self> {
        let bytes = command_line.as_bytes();
        if bytes.ends_with(&[0]) {
            new_boxed(&[bytes])
        } else {
            new_boxed(&[bytes, &[0]])
        }
    }

    /// Reads the command line of the kernel as Rust string slice without
    /// the null-byte.
    ///
    /// For example, this returns `"console=ttyS0"`.if the GRUB config
    /// contains  `"multiboot2 /mykernel console=ttyS0"`.
    ///
    /// If the function returns `Err` then perhaps the memory is invalid.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use multiboot2::{BootInformation, BootInformationHeader};
    /// # let ptr = 0xdeadbeef as *const BootInformationHeader;
    /// # let boot_info = unsafe { BootInformation::load(ptr).unwrap() };
    /// if let Some(tag) = boot_info.command_line_tag() {
    ///     let command_line = tag.cmdline();
    ///     assert_eq!(Ok("/bootarg"), command_line);
    /// }
    /// ```
    pub fn cmdline(&self) -> Result<&str, StringError> {
        parse_slice_as_string(&self.cmdline)
    }
}

impl Debug for CommandLineTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CommandLineTag")
            .field("typ", &self.header.typ)
            .field("size", &self.header.size)
            .field("cmdline", &self.cmdline())
            .finish()
    }
}

impl TagTrait for CommandLineTag {
    const ID: TagType = TagType::Cmdline;

    fn dst_len(header: &TagHeader) -> usize {
        assert!(header.size as usize >= METADATA_SIZE);
        header.size as usize - METADATA_SIZE
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tag::{GenericTag, TagBytesRef};
    use crate::test_util::AlignedBytes;

    #[rustfmt::skip]
    fn get_bytes() -> AlignedBytes<16> {
        AlignedBytes::new([
            TagType::Cmdline.val() as u8, 0, 0, 0,
            14, 0, 0, 0,
            b'h', b'e', b'l', b'l', b'o',  b'\0',
            /* padding */
            0, 0
        ])
    }

    /// Tests to parse a string with a terminating null byte from the tag (as the spec defines).
    #[test]
    fn test_parse_str() {
        let bytes = get_bytes();
        let bytes = TagBytesRef::try_from(&bytes[..]).unwrap();
        let tag = GenericTag::ref_from(bytes);
        let tag = tag.cast::<CommandLineTag>();
        assert_eq!(tag.header.typ, TagType::Cmdline);
        assert_eq!(tag.cmdline(), Ok("hello"));
    }

    /// Test to generate a tag from a given string.
    #[test]
    #[cfg(feature = "builder")]
    fn test_build_str() {
        let tag = CommandLineTag::new("hello");
        let bytes = tag.as_bytes();
        assert_eq!(bytes, &get_bytes()[..tag.size()]);
        assert_eq!(tag.cmdline(), Ok("hello"));

        // test also some bigger message
        let tag = CommandLineTag::new("AbCdEfGhUjK YEAH");
        assert_eq!(tag.cmdline(), Ok("AbCdEfGhUjK YEAH"));
        let tag = CommandLineTag::new("AbCdEfGhUjK YEAH".repeat(42).as_str());
        assert_eq!(tag.cmdline(), Ok("AbCdEfGhUjK YEAH".repeat(42).as_str()));
    }
}
