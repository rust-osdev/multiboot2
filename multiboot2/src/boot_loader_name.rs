use crate::{Tag, TagTrait, TagTypeId};
use core::fmt::{Debug, Formatter};
use core::mem::size_of;
use core::str::Utf8Error;

#[cfg(feature = "builder")]
use {
    crate::builder::boxed_dst_tag, crate::builder::traits::StructAsBytes, crate::TagType,
    alloc::boxed::Box, alloc::vec::Vec,
};

const METADATA_SIZE: usize = size_of::<TagTypeId>() + size_of::<u32>();

/// The bootloader name tag.
#[derive(ptr_meta::Pointee, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct BootLoaderNameTag {
    typ: TagTypeId,
    size: u32,
    /// Null-terminated UTF-8 string
    name: [u8],
}

impl BootLoaderNameTag {
    #[cfg(feature = "builder")]
    pub fn new(name: &str) -> Box<Self> {
        let mut bytes: Vec<_> = name.bytes().collect();
        bytes.push(0);
        boxed_dst_tag(TagType::BootLoaderName, &bytes)
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
    /// # let boot_info = unsafe { multiboot2::load(0xdeadbeef).unwrap() };
    /// if let Some(tag) = boot_info.boot_loader_name_tag() {
    ///     assert_eq!(Ok("GRUB 2.02~beta3-5"), tag.name());
    /// }
    /// ```
    pub fn name(&self) -> Result<&str, Utf8Error> {
        Tag::get_dst_str_slice(&self.name)
    }
}

impl Debug for BootLoaderNameTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("BootLoaderNameTag")
            .field("typ", &{ self.typ })
            .field("size", &{ self.size })
            .field("name", &self.name())
            .finish()
    }
}

impl TagTrait for BootLoaderNameTag {
    fn dst_size(base_tag: &Tag) -> usize {
        assert!(base_tag.size as usize >= METADATA_SIZE);
        base_tag.size as usize - METADATA_SIZE
    }
}

#[cfg(feature = "builder")]
impl StructAsBytes for BootLoaderNameTag {
    fn byte_size(&self) -> usize {
        self.size.try_into().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::{BootLoaderNameTag, Tag, TagType};

    const MSG: &str = "hello";

    /// Returns the tag structure in bytes in native endian format.
    fn get_bytes() -> std::vec::Vec<u8> {
        // size is: 4 bytes for tag + 4 bytes for size + length of null-terminated string
        let size = (4 + 4 + MSG.as_bytes().len() + 1) as u32;
        [
            &((TagType::BootLoaderName.val()).to_le_bytes()),
            &size.to_le_bytes(),
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
        let tag = unsafe { &*tag.as_ptr().cast::<Tag>() };
        let tag = tag.cast_tag::<BootLoaderNameTag>();
        assert_eq!({ tag.typ }, TagType::BootLoaderName);
        assert_eq!(tag.name().expect("must be valid UTF-8"), MSG);
    }

    /// Test to generate a tag from a given string.
    #[test]
    #[cfg(feature = "builder")]
    fn test_build_str() {
        use crate::builder::traits::StructAsBytes;

        let tag = BootLoaderNameTag::new(MSG);
        let bytes = tag.struct_as_bytes();
        assert_eq!(bytes, get_bytes());
    }
}
