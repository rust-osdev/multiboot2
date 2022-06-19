use crate::TagType;
use core::str::Utf8Error;

/// This tag contains the name of the bootloader that is booting the kernel.
///
/// The name is a normal C-style UTF-8 zero-terminated string that can be
/// obtained via the `name` method.
#[derive(Clone, Copy, Debug)]
#[repr(C, packed)] // only repr(C) would add unwanted padding before first_section
pub struct BootLoaderNameTag {
    typ: TagType,
    size: u32,
    /// Null-terminated UTF-8 string
    string: u8,
}

impl BootLoaderNameTag {
    /// Read the name of the bootloader that is booting the kernel.
    /// This is an null-terminated UTF-8 string. If this returns `Err` then perhaps the memory
    /// is invalid or the bootloader doesn't follow the spec.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// if let Some(tag) = boot_info.boot_loader_name_tag() {
    ///     let name = tag.name();
    ///     assert_eq!("GRUB 2.02~beta3-5", name);
    /// }
    /// ```
    pub fn name(&self) -> Result<&str, Utf8Error> {
        use core::{mem, slice, str};
        let strlen = self.size as usize - mem::size_of::<BootLoaderNameTag>();
        let bytes = unsafe { slice::from_raw_parts((&self.string) as *const u8, strlen) };
        str::from_utf8(bytes)
    }
}
