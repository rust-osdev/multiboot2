//! Module for [`BytesRef`].

use crate::{Header, MemoryError, ALIGNMENT};
use core::marker::PhantomData;
use core::mem;
use core::ops::Deref;

/// Wraps a byte slice representing a Multiboot2 structure including an optional
/// terminating padding, if necessary. It guarantees that the memory
/// requirements promised in the crates description are respected.
#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct BytesRef<'a, H: Header> {
    bytes: &'a [u8],
    // Ensure that consumers can rely on the size properties for `H` that
    // already have been verified when this type was constructed.
    _h: PhantomData<H>,
}

impl<'a, H: Header> TryFrom<&'a [u8]> for BytesRef<'a, H> {
    type Error = MemoryError;

    fn try_from(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        if bytes.len() < mem::size_of::<H>() {
            return Err(MemoryError::MinLengthNotSatisfied);
        }
        // Doesn't work as expected: if align_of_val(&value[0]) < ALIGNMENT {
        if bytes.as_ptr().align_offset(ALIGNMENT) != 0 {
            return Err(MemoryError::WrongAlignment);
        }
        let padding_bytes = bytes.len() % ALIGNMENT;
        if padding_bytes != 0 {
            return Err(MemoryError::MissingPadding);
        }
        Ok(Self {
            bytes,
            _h: PhantomData,
        })
    }
}

impl<'a, H: Header> Deref for BytesRef<'a, H> {
    type Target = &'a [u8];

    fn deref(&self) -> &Self::Target {
        &self.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{AlignedBytes, DummyTestHeader};

    #[test]
    fn test_bytes_ref() {
        let empty: &[u8] = &[];
        assert_eq!(
            BytesRef::<'_, DummyTestHeader>::try_from(empty),
            Err(MemoryError::MinLengthNotSatisfied)
        );

        let slice = &[0_u8, 1, 2, 3, 4, 5, 6];
        assert_eq!(
            BytesRef::<'_, DummyTestHeader>::try_from(&slice[..]),
            Err(MemoryError::MinLengthNotSatisfied)
        );

        let slice = AlignedBytes([0_u8, 1, 2, 3, 4, 5, 6, 7, 0, 0, 0]);
        // Guaranteed wrong alignment
        let unaligned_slice = &slice[3..];
        assert_eq!(
            BytesRef::<'_, DummyTestHeader>::try_from(unaligned_slice),
            Err(MemoryError::WrongAlignment)
        );

        let slice = AlignedBytes([0_u8, 1, 2, 3, 4, 5, 6, 7]);
        let slice = &slice[..];
        assert_eq!(
            BytesRef::try_from(slice),
            Ok(BytesRef {
                bytes: slice,
                _h: PhantomData::<DummyTestHeader>
            })
        );
    }
}
