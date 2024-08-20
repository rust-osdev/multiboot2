use crate::{HeaderTagFlag, HeaderTagHeader};
use crate::{HeaderTagType, MbiTagTypeId};
use core::fmt;
use core::fmt::{Debug, Formatter};
use core::mem;
#[cfg(feature = "builder")]
use multiboot2_common::new_boxed;
use multiboot2_common::{MaybeDynSized, Tag};
#[cfg(feature = "builder")]
use {
    alloc::boxed::Box,
    core::{ptr, slice},
};

/// Specifies what specific tag types the bootloader should provide
/// inside the mbi.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, ptr_meta::Pointee)]
#[repr(C, align(8))]
pub struct InformationRequestHeaderTag {
    header: HeaderTagHeader,
    requests: [MbiTagTypeId],
}

impl InformationRequestHeaderTag {
    /// Creates a new object.
    #[cfg(feature = "builder")]
    #[must_use]
    pub fn new(flags: HeaderTagFlag, requests: &[MbiTagTypeId]) -> Box<Self> {
        let header = HeaderTagHeader::new(HeaderTagType::InformationRequest, flags, 0);
        let requests = unsafe {
            let ptr = ptr::addr_of!(*requests);
            slice::from_raw_parts(ptr.cast::<u8>(), mem::size_of_val(requests))
        };
        new_boxed(header, &[requests])
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

    /// Returns the requests as array
    #[must_use]
    pub const fn requests(&self) -> &[MbiTagTypeId] {
        &self.requests
    }
}

impl Debug for InformationRequestHeaderTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("InformationRequestHeaderTag")
            .field("type", &self.typ())
            .field("flags", &self.flags())
            .field("size", &self.size())
            .field("requests", &self.requests())
            .finish()
    }
}

impl MaybeDynSized for InformationRequestHeaderTag {
    type Header = HeaderTagHeader;

    const BASE_SIZE: usize = mem::size_of::<HeaderTagHeader>();

    fn dst_len(header: &Self::Header) -> Self::Metadata {
        let dst_size = header.size() as usize - Self::BASE_SIZE;
        assert_eq!(dst_size % mem::size_of::<MbiTagTypeId>(), 0);
        dst_size / mem::size_of::<MbiTagTypeId>()
    }
}

impl Tag for InformationRequestHeaderTag {
    type IDType = HeaderTagType;
    const ID: HeaderTagType = HeaderTagType::InformationRequest;
}

#[cfg(test)]
#[cfg(feature = "builder")]
mod tests {
    use super::*;
    use crate::MbiTagType;

    #[test]
    fn creation() {
        // Main objective here is to satisfy Miri.
        let _ir = InformationRequestHeaderTag::new(
            HeaderTagFlag::Optional,
            &[
                MbiTagType::Cmdline.into(),
                MbiTagType::BootLoaderName.into(),
                MbiTagType::Module.into(),
                MbiTagType::BasicMeminfo.into(),
                MbiTagType::Bootdev.into(),
                MbiTagType::Mmap.into(),
                MbiTagType::Vbe.into(),
                MbiTagType::Framebuffer.into(),
                MbiTagType::ElfSections.into(),
                MbiTagType::Apm.into(),
                MbiTagType::Efi32.into(),
                MbiTagType::Efi64.into(),
                MbiTagType::Smbios.into(),
                MbiTagType::AcpiV1.into(),
                MbiTagType::AcpiV2.into(),
                MbiTagType::Network.into(),
                MbiTagType::EfiMmap.into(),
                MbiTagType::EfiBs.into(),
                MbiTagType::Efi32Ih.into(),
                MbiTagType::Efi64Ih.into(),
                MbiTagType::LoadBaseAddr.into(),
                MbiTagType::Custom(0x1337).into(),
            ],
        );
    }
}
