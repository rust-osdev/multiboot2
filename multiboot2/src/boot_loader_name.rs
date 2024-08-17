//! Module for [`BootLoaderNameTag`].

use crate::tag::TagHeader;
use crate::{new_boxed, parse_slice_as_string, StringError, TagTrait, TagType};
#[cfg(feature = "builder")]
use alloc::boxed::Box;
use core::fmt::{Debug, Formatter};
use core::mem;

const METADATA_SIZE: usize = mem::size_of::<TagHeader>();

/// The bootloader name tag.
#[derive(ptr_meta::Pointee, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C, align(8))]
pub struct BootLoaderNameTag {
    header: TagHeader,
    /// Null-terminated UTF-8 string
    name: [u8],
}

impl BootLoaderNameTag {
    /// Constructs a new tag.
    #[cfg(feature = "builder")]
    #[must_use]
    pub fn new(name: &str) -> Box<Self> {
        let bytes = name.as_bytes();
        if bytes.ends_with(&[0]) {
            new_boxed(&[bytes])
        } else {
            new_boxed(&[bytes, &[0]])
        }
    }

    /// Returns the underlying [`TagType`].
    #[must_use]
    pub fn typ(&self) -> TagType {
        self.header.typ.into()
    }

    /// Returns the underlying tag size.
    #[must_use]
    pub const fn size(&self) -> usize {
        self.header.size as usize
    }

    /// Reads the name of the bootloader that is booting the kernel as Rust
    /// string slice without the null-byte.
    ///
    /// For example, this returns `"GRUB 2.02~beta3-5"`.
    ///
    /// If the function returns `Err` then perhaps the memory is invalid.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use multiboot2::{BootInformation, BootInformationHeader};
    /// # let ptr = 0xdeadbeef as *const BootInformationHeader;
    /// # let boot_info = unsafe { BootInformation::load(ptr).unwrap() };
    /// if let Some(tag) = boot_info.boot_loader_name_tag() {
    ///     assert_eq!(Ok("GRUB 2.02~beta3-5"), tag.name());
    /// }
    /// ```
    pub fn name(&self) -> Result<&str, StringError> {
        parse_slice_as_string(&self.name)
    }
}

impl Debug for BootLoaderNameTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("BootLoaderNameTag")
            .field("typ", &self.header.typ)
            .field("size", &self.header.size)
            .field("name", &self.name())
            .finish()
    }
}

impl TagTrait for BootLoaderNameTag {
    const ID: TagType = TagType::BootLoaderName;

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
            TagType::BootLoaderName.val() as u8, 0, 0, 0,
            14, 0, 0, 0,
            b'h', b'e', b'l', b'l', b'o', b'\0',
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
        let tag = tag.cast::<BootLoaderNameTag>();
        assert_eq!(tag.header.typ, TagType::BootLoaderName);
        assert_eq!(tag.name(), Ok("hello"));
    }

    /// Test to generate a tag from a given string.
    #[test]
    #[cfg(feature = "builder")]
    fn test_build_str() {
        let tag = BootLoaderNameTag::new("hello");
        let bytes = tag.as_bytes();
        assert_eq!(bytes, &get_bytes()[..tag.size()]);
        assert_eq!(tag.name(), Ok("hello"));

        // test also some bigger message
        let tag = BootLoaderNameTag::new("AbCdEfGhUjK YEAH");
        assert_eq!(tag.name(), Ok("AbCdEfGhUjK YEAH"));
        let tag = BootLoaderNameTag::new("AbCdEfGhUjK YEAH".repeat(42).as_str());
        assert_eq!(tag.name(), Ok("AbCdEfGhUjK YEAH".repeat(42).as_str()));
    }
}
