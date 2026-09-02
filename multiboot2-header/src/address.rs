use crate::{HeaderTagFlag, HeaderTagHeader, HeaderTagType};
use multiboot2_common::{MaybeDynSized, Tag};

/// Binary address information for non-ELF images.
///
/// This information does not need to be provided if the kernel image is in ELF
/// format, but it must be provided if the image is in a.out format or in some
/// other format. Required for legacy boot (BIOS).
/// Determines load addresses.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C, align(8))]
pub struct AddressHeaderTag {
    header: HeaderTagHeader,
    /// Address corresponding to the beginning of the Multiboot2 header.
    ///
    /// This is the physical memory location at which the magic value is
    /// supposed to be loaded. It synchronizes the mapping between OS image
    /// offsets and physical memory addresses.
    header_addr: u32,
    /// Physical address of the beginning of the text segment.
    ///
    /// The offset in the OS image file at which to start loading is the offset
    /// at which the header was found, minus `header_addr - load_addr`.
    /// `load_addr` must be less than or equal to `header_addr`.
    ///
    /// The special value -1 means that the file must be loaded from its
    /// beginning.
    load_addr: u32,
    /// Physical address of the end of the data segment.
    ///
    /// `load_end_addr - load_addr` specifies how much data to load. This
    /// implies that the text and data segments must be consecutive in the OS
    /// image, as they are in existing a.out executable formats. If this field
    /// is zero, the bootloader assumes that the text and data segments occupy
    /// the whole OS image file.
    load_end_addr: u32,
    /// Physical address of the end of the BSS segment.
    ///
    /// The bootloader initializes this area to zero and reserves its memory
    /// to avoid placing boot modules and other operating-system data there. If
    /// this field is zero, the bootloader assumes that no BSS segment exists.
    bss_end_addr: u32,
}

const _: () = assert!(size_of::<AddressHeaderTag>() == 2 + 2 + 4 + 4 + 4 + 4 + 4);

impl AddressHeaderTag {
    /// Constructs a new tag.
    #[must_use]
    pub const fn new(
        flags: HeaderTagFlag,
        header_addr: u32,
        load_addr: u32,
        load_end_addr: u32,
        bss_end_addr: u32,
    ) -> Self {
        let header = HeaderTagHeader::new(HeaderTagType::Address, flags, size_of::<Self>() as u32);
        Self {
            header,
            header_addr,
            load_addr,
            load_end_addr,
            bss_end_addr,
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

    /// Returns the header address.
    #[must_use]
    pub const fn header_addr(&self) -> u32 {
        self.header_addr
    }

    /// Returns the load begin address.
    #[must_use]
    pub const fn load_addr(&self) -> u32 {
        self.load_addr
    }

    /// Returns the load end address.
    #[must_use]
    pub const fn load_end_addr(&self) -> u32 {
        self.load_end_addr
    }

    /// Returns the bss end address.
    #[must_use]
    pub const fn bss_end_addr(&self) -> u32 {
        self.bss_end_addr
    }
}

// SAFETY: The tag is repr(C) with the header as first field, any bit
// pattern is valid, and `BASE_SIZE`/`dst_len` match the ABI.
unsafe impl MaybeDynSized for AddressHeaderTag {
    type Header = HeaderTagHeader;

    const BASE_SIZE: usize = size_of::<Self>();
}

impl Tag for AddressHeaderTag {
    type IDType = HeaderTagType;
    const ID: HeaderTagType = HeaderTagType::Address;
}
