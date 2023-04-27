use crate::TagTrait;
use crate::{Tag, TagTypeId};
use core::fmt::{Debug, Formatter};
use core::str::Utf8Error;

/// The bootloader name tag.
#[derive(ptr_meta::Pointee)]
#[repr(C, packed)] // only repr(C) would add unwanted padding before first_section
pub struct BootLoaderNameTag {
    typ: TagTypeId,
    size: u32,
    /// Null-terminated UTF-8 string
    name: [u8],
}

impl BootLoaderNameTag {
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
        // The size of the sized portion of the bootloader name tag.
        let tag_base_size = 8;
        assert!(base_tag.size >= 8);
        base_tag.size as usize - tag_base_size
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
            &((TagType::BootLoaderName.val()).to_ne_bytes()),
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
        let tag = unsafe { &*tag.as_ptr().cast::<Tag>() };
        let tag = tag.cast_tag::<BootLoaderNameTag>();
        assert_eq!({ tag.typ }, TagType::BootLoaderName);
        assert_eq!(tag.name().expect("must be valid UTF-8"), MSG);
    }
}
