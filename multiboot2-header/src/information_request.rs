use crate::{HeaderTagFlag, HeaderTagHeader, HeaderTagType};
use crate::{MbiTagType, MbiTagTypeRaw};
use core::fmt;
use core::fmt::{Debug, Formatter};
#[cfg(feature = "builder")]
use multiboot2_common::new_boxed;
use multiboot2_common::{MaybeDynSized, Tag};
#[cfg(feature = "builder")]
use {alloc::boxed::Box, alloc::vec::Vec};

/// Specifies which tag types the bootloader should provide
/// inside the mbi.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, ptr_meta::Pointee)]
#[repr(C, align(8))]
pub struct InformationRequestHeaderTag {
    header: HeaderTagHeader,
    requests: [MbiTagTypeRaw],
}

impl InformationRequestHeaderTag {
    /// Creates a new object.
    #[cfg(feature = "builder")]
    #[must_use]
    pub fn new(flags: HeaderTagFlag, requests: &[MbiTagType]) -> Box<Self> {
        let header = HeaderTagHeader::new(HeaderTagType::InformationRequest, flags, 0);
        let requests = requests
            .iter()
            .flat_map(|request| request.val().to_ne_bytes())
            .collect::<Vec<_>>();
        new_boxed(header, &[requests.as_slice()])
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

    /// Returns an iterator over the requested tag types.
    pub fn requests(&self) -> impl Iterator<Item = MbiTagType> + '_ {
        self.requests.iter().map(|&raw| MbiTagType::from(raw))
    }
}

/// Debug-formats the requests of an [`InformationRequestHeaderTag`] as a list
/// of [`MbiTagType`].
struct RequestsDebug<'a>(&'a InformationRequestHeaderTag);

impl Debug for RequestsDebug<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.0.requests()).finish()
    }
}

impl Debug for InformationRequestHeaderTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("InformationRequestHeaderTag")
            .field("type", &self.typ())
            .field("flags", &self.flags())
            .field("size", &self.size())
            .field("requests", &RequestsDebug(self))
            .finish()
    }
}

impl MaybeDynSized for InformationRequestHeaderTag {
    type Header = HeaderTagHeader;

    const BASE_SIZE: usize = size_of::<HeaderTagHeader>();

    fn dst_len(header: &Self::Header) -> Self::Metadata {
        assert!(header.size() as usize >= Self::BASE_SIZE);
        let dst_size = header.size() as usize - Self::BASE_SIZE;
        assert_eq!(dst_size % size_of::<MbiTagTypeRaw>(), 0);
        dst_size / size_of::<MbiTagTypeRaw>()
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

    #[test]
    #[should_panic]
    fn dst_len_rejects_undersized_header() {
        let header = HeaderTagHeader::new(
            HeaderTagType::InformationRequest,
            HeaderTagFlag::Optional,
            4,
        );
        let _ = <InformationRequestHeaderTag as MaybeDynSized>::dst_len(&header);
    }

    #[test]
    fn creation() {
        let requests = [
            MbiTagType::Cmdline,
            MbiTagType::BootLoaderName,
            MbiTagType::Module,
            MbiTagType::BasicMeminfo,
            MbiTagType::Bootdev,
            MbiTagType::Mmap,
            MbiTagType::Vbe,
            MbiTagType::Framebuffer,
            MbiTagType::ElfSections,
            MbiTagType::Apm,
            MbiTagType::Efi32,
            MbiTagType::Efi64,
            MbiTagType::Smbios,
            MbiTagType::AcpiV1,
            MbiTagType::AcpiV2,
            MbiTagType::Network,
            MbiTagType::EfiMmap,
            MbiTagType::EfiBs,
            MbiTagType::Efi32Ih,
            MbiTagType::Efi64Ih,
            MbiTagType::LoadBaseAddr,
            MbiTagType::Custom(0x1337),
        ];
        // Statement also is a good test for Miri.
        let ir = InformationRequestHeaderTag::new(HeaderTagFlag::Optional, &requests);
        assert!(ir.requests().eq(requests.iter().copied()));
    }
}
