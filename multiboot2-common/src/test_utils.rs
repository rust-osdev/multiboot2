//! Various test utilities.

#![allow(missing_docs)]

use crate::{Header, MaybeDynSized, Tag};
use core::borrow::Borrow;
use core::mem;
use core::ops::Deref;

/// Helper to 8-byte align the underlying bytes, as mandated in the Multiboot2
/// spec. With this type, one can create manual and raw Multiboot2 boot
/// information or just the bytes for simple tags, in a manual and raw approach.
#[derive(Debug)]
#[repr(C, align(8))]
pub struct AlignedBytes<const N: usize>(pub [u8; N]);

impl<const N: usize> AlignedBytes<N> {
    /// Creates a new type.
    #[must_use]
    pub const fn new(bytes: [u8; N]) -> Self {
        Self(bytes)
    }
}

impl<const N: usize> Borrow<[u8; N]> for AlignedBytes<N> {
    fn borrow(&self) -> &[u8; N] {
        &self.0
    }
}

impl<const N: usize> Borrow<[u8]> for AlignedBytes<N> {
    fn borrow(&self) -> &[u8] {
        &self.0
    }
}

impl<const N: usize> Deref for AlignedBytes<N> {
    type Target = [u8; N];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Dummy test header.
#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(C, align(8))]
pub struct DummyTestHeader {
    typ: u32,
    size: u32,
}

impl DummyTestHeader {
    #[must_use]
    pub const fn new(typ: u32, size: u32) -> Self {
        Self { typ, size }
    }

    #[must_use]
    pub const fn typ(&self) -> u32 {
        self.typ
    }

    #[must_use]
    pub const fn size(&self) -> u32 {
        self.size
    }
}

impl Header for DummyTestHeader {
    fn payload_len(&self) -> usize {
        self.size as usize - mem::size_of::<Self>()
    }

    fn set_size(&mut self, total_size: usize) {
        self.size = total_size as u32;
    }
}

#[derive(Debug, PartialEq, Eq, ptr_meta::Pointee)]
#[repr(C, align(8))]
pub struct DummyDstTag {
    header: DummyTestHeader,
    payload: [u8],
}

impl DummyDstTag {
    const BASE_SIZE: usize = mem::size_of::<DummyTestHeader>();

    #[must_use]
    pub const fn header(&self) -> &DummyTestHeader {
        &self.header
    }

    #[must_use]
    pub const fn payload(&self) -> &[u8] {
        &self.payload
    }
}

impl MaybeDynSized for DummyDstTag {
    type Header = DummyTestHeader;

    const BASE_SIZE: usize = mem::size_of::<DummyTestHeader>();

    fn dst_len(header: &Self::Header) -> Self::Metadata {
        header.size as usize - Self::BASE_SIZE
    }
}

impl Tag for DummyDstTag {
    type IDType = u32;
    const ID: Self::IDType = 42;
}

#[cfg(test)]
mod tests {
    use core::mem;
    use core::ptr::addr_of;

    use crate::ALIGNMENT;

    use super::*;

    #[test]
    fn abi() {
        assert_eq!(mem::align_of::<AlignedBytes<0>>(), ALIGNMENT);

        let bytes = AlignedBytes([0]);
        assert_eq!(bytes.as_ptr().align_offset(8), 0);
        assert_eq!((addr_of!(bytes[0])).align_offset(8), 0);

        let bytes = AlignedBytes([0, 1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(bytes.as_ptr().align_offset(8), 0);
        assert_eq!((addr_of!(bytes[0])).align_offset(8), 0);

        let bytes = AlignedBytes([0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        assert_eq!(bytes.as_ptr().align_offset(8), 0);
        assert_eq!((addr_of!(bytes[0])).align_offset(8), 0);
        assert_eq!((addr_of!(bytes[7])).align_offset(8), 1);
        assert_eq!((addr_of!(bytes[8])).align_offset(8), 0);
        assert_eq!((addr_of!(bytes[9])).align_offset(8), 7);
    }
}
