use crate::{HeaderTagFlag, HeaderTagHeader, HeaderTagType};
use multiboot2_common::{MaybeDynSized, Tag};

multiboot2_common::raw_type! {
    /// ABI compatible representation of the console flags of the
    /// [`ConsoleHeaderTag`].
    ///
    /// This type matches the binary representation (`u32`).
    pub struct ConsoleHeaderTagFlagsRaw(u32);

    /// The console flags of the [`ConsoleHeaderTag`].
    ///
    /// This is a higher level abstraction for [`ConsoleHeaderTagFlagsRaw`].
    pub enum ConsoleHeaderTagFlags {
        /// At least one of the consoles supported by the bootloader must be
        /// present and information about it must be available in the boot
        /// information.
        ConsoleRequired = 1,
        /// The OS image has EGA text support.
        EgaTextSupported = 2,
    }
}

/// Tells that a console must be available in MBI.
/// Only relevant for legacy BIOS.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C, align(8))]
pub struct ConsoleHeaderTag {
    header: HeaderTagHeader,
    console_flags: ConsoleHeaderTagFlagsRaw,
}

impl ConsoleHeaderTag {
    /// Constructs a new tag.
    #[must_use]
    pub const fn new(flags: HeaderTagFlag, console_flags: ConsoleHeaderTagFlags) -> Self {
        let header =
            HeaderTagHeader::new(HeaderTagType::ConsoleFlags, flags, Self::BASE_SIZE as u32);
        Self {
            header,
            console_flags: ConsoleHeaderTagFlagsRaw::new(console_flags.val()),
        }
    }

    /// Returns the [`HeaderTagType`].
    #[must_use]
    pub const fn typ(&self) -> HeaderTagType {
        self.header.typ()
    }

    /// Returns the [`HeaderTagFlag`]s.
    #[must_use]
    pub const fn flags(&self) -> HeaderTagFlag {
        self.header.flags()
    }

    /// Returns the size.
    #[must_use]
    pub const fn size(&self) -> u32 {
        self.header.size()
    }

    /// Returns the [`ConsoleHeaderTagFlags`].
    #[must_use]
    pub const fn console_flags(&self) -> ConsoleHeaderTagFlags {
        ConsoleHeaderTagFlags::from_val(self.console_flags.get())
    }
}

// SAFETY: The tag is repr(C) with the header as first field, any bit
// pattern is valid, and `BASE_SIZE`/`dst_len` match the ABI.
unsafe impl MaybeDynSized for ConsoleHeaderTag {
    type Header = HeaderTagHeader;

    const BASE_SIZE: usize = size_of::<HeaderTagHeader>() + size_of::<u32>();
}

impl Tag for ConsoleHeaderTag {
    type IDType = HeaderTagType;
    const ID: HeaderTagType = HeaderTagType::ConsoleFlags;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GenericHeaderTag;
    use core::borrow::Borrow;
    use multiboot2_common::test_utils::AlignedBytes;

    /// The console flag values must match the values mandated by the
    /// specification.
    #[test]
    fn console_flags_match_spec_values() {
        assert_eq!(ConsoleHeaderTagFlags::ConsoleRequired.val(), 1);
        assert_eq!(ConsoleHeaderTagFlags::EgaTextSupported.val(), 2);
    }

    /// A tag with a console flags value unknown to the specification must be
    /// parsable without undefined behavior.
    #[test]
    fn unknown_console_flags_are_not_ub() {
        #[rustfmt::skip]
        let bytes = AlignedBytes::new([
            /* typ = console flags */
            4, 0,
            /* flags */
            0, 0,
            /* size */
            12, 0, 0, 0,
            /* console_flags = 3 (unknown) */
            3, 0, 0, 0,
            /* padding to alignment */
            0, 0, 0, 0,
        ]);
        let tag = GenericHeaderTag::ref_from_slice(bytes.borrow())
            .unwrap()
            .cast::<ConsoleHeaderTag>();

        assert_eq!(tag.console_flags(), ConsoleHeaderTagFlags::Custom(3));
    }
}
