use crate::{HeaderTagFlag, HeaderTagType, StructAsBytes};
use core::mem::size_of;

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum ConsoleHeaderTagFlags {
    /// Console required.
    ConsoleRequired = 0,
    /// EGA text support.
    EgaTextSupported = 1,
}

/// Tells that a console must be available in MBI.
/// Only relevant for legacy BIOS.
#[derive(Copy, Clone, Debug)]
#[repr(C, packed(8))]
pub struct ConsoleHeaderTag {
    typ: HeaderTagType,
    flags: HeaderTagFlag,
    size: u32,
    console_flags: ConsoleHeaderTagFlags,
}

impl ConsoleHeaderTag {
    pub const fn new(flags: HeaderTagFlag, console_flags: ConsoleHeaderTagFlags) -> Self {
        ConsoleHeaderTag {
            typ: HeaderTagType::ConsoleFlags,
            flags,
            size: size_of::<Self>() as u32,
            console_flags,
        }
    }

    pub fn typ(&self) -> HeaderTagType {
        self.typ
    }
    pub fn flags(&self) -> HeaderTagFlag {
        self.flags
    }
    pub fn size(&self) -> u32 {
        self.size
    }
    pub fn console_flags(&self) -> ConsoleHeaderTagFlags {
        self.console_flags
    }
}

impl StructAsBytes for ConsoleHeaderTag {}

#[cfg(test)]
mod tests {
    use crate::{ConsoleHeaderTag, ConsoleHeaderTagFlags, HeaderTagFlag, HeaderTagType};
    use std::mem::size_of_val;

    /// Checks if rust aligns the type correctly and still "pack" all properties.
    /// This test is necessary, because Rust doesn't support "packed" together with "align()" yet.
    /// It seems like "packed(N)" does the right thing tho.
    ///
    /// This test is representative for all header tags, because all use the "packed(8)" attribute.
    #[test]
    fn test_alignment_and_size() {
        let tag = ConsoleHeaderTag::new(
            HeaderTagFlag::Required,
            ConsoleHeaderTagFlags::ConsoleRequired,
        );
        let ptr = get_ptr!(tag, ConsoleHeaderTag);
        let is_aligned = ptr % 8 == 0;
        assert!(is_aligned);
        // 2x u16, 2x u32
        assert_eq!(2 + 2 + 4 + 4, size_of_val(&tag));

        assert_eq!(ptr + 0, get_field_ptr!(tag, typ, HeaderTagType));
        assert_eq!(ptr + 2, get_field_ptr!(tag, flags, HeaderTagFlag));
        assert_eq!(ptr + 4, get_field_ptr!(tag, size, u32));
        assert_eq!(
            ptr + 8,
            get_field_ptr!(tag, console_flags, ConsoleHeaderTagFlags)
        );
    }
}
