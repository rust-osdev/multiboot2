//! Various test utilities.

use crate::ALIGNMENT;
use core::borrow::Borrow;
use core::ops::Deref;

/// Helper to 8-byte align the underlying bytes, as mandated in the Multiboot2
/// spec. With this type, one can create manual and raw Multiboot2 boot
/// information or just the bytes for simple tags, in a manual and raw approach.
#[cfg(test)]
#[repr(C, align(8))]
pub struct AlignedBytes<const N: usize>(pub [u8; N]);

impl<const N: usize> AlignedBytes<N> {
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

// The tests down below are all Miri-approved.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::tag::TagBytesRef;
    use core::mem;
    use core::ptr::addr_of;

    #[test]
    fn abi() {
        let bytes = AlignedBytes([0]);
        assert_eq!(mem::align_of_val(&bytes), ALIGNMENT);
        assert_eq!(bytes.as_ptr().align_offset(8), 0);
        assert_eq!((addr_of!(bytes[0])).align_offset(8), 0);

        let bytes = AlignedBytes([0, 1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(mem::align_of_val(&bytes), ALIGNMENT);
        assert_eq!(bytes.as_ptr().align_offset(8), 0);
        assert_eq!((addr_of!(bytes[0])).align_offset(8), 0);

        let bytes = AlignedBytes([0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        assert_eq!(mem::align_of_val(&bytes), ALIGNMENT);
        assert_eq!(bytes.as_ptr().align_offset(8), 0);
        assert_eq!((addr_of!(bytes[0])).align_offset(8), 0);
        assert_eq!((addr_of!(bytes[7])).align_offset(8), 1);
        assert_eq!((addr_of!(bytes[8])).align_offset(8), 0);
        assert_eq!((addr_of!(bytes[9])).align_offset(8), 7);
    }

    #[test]
    fn compatible_with_tag_bytes_ref_type() {
        #[rustfmt::skip]
        let bytes = AlignedBytes(
            [
                /* tag id */
                0, 0, 0, 0,
                /* size */
                14, 0, 0, 0,
                /* arbitrary payload */
                1, 2, 3, 4,
                5, 6,
                /* padding */
                0, 0,
            ]
        );
        let _a = TagBytesRef::try_from(&bytes[..]).unwrap();
    }
}
