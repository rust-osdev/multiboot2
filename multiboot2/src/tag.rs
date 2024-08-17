//! Module for the base tag definitions and helper types.
//!
//! The relevant exports of this module are [`TagHeader`], [`GenericTag`], and
//! [`TagIter`].
//!
//! The (internal) workflow to parse a tag from bytes is the following:
//! - `&[u8]` --> [`TagBytesRef`]
//! - [`TagBytesRef`] --> [`TagHeader`]
//! - [`TagBytesRef`] + [`TagHeader`] --> [`GenericTag`]
//! - [`GenericTag`] --> cast to desired tag

use crate::util::increase_to_alignment;
use crate::{TagTrait, TagType, TagTypeId, ALIGNMENT};
use core::fmt::{Debug, Formatter};
use core::mem;
use core::ops::Deref;
use core::ptr;

/// The common header that all tags have in common. This type is ABI compatible.
/// It is the sized counterpart of [`GenericTag`].
///
/// Not to be confused with Multiboot header tags, which are something
/// different.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(C, align(8))] // Alignment also propagates to all tag types using this.
pub struct TagHeader {
    /// The ABI-compatible [`TagType`].
    pub typ: TagTypeId, /* u32 */
    /// The total size of the tag including the header.
    pub size: u32,
    // Followed by optional additional tag specific fields.
}

impl TagHeader {
    /// Creates a new header.
    pub fn new(typ: impl Into<TagTypeId>, size: u32) -> Self {
        Self {
            typ: typ.into(),
            size,
        }
    }
}

/// Wraps a byte slice representing a tag, but guarantees that the memory
/// requirements are fulfilled.
///
/// This is the only type that can be used to construct a [`GenericTag`].
///
/// The main reason for this dedicated type is to create fine-grained unit-tests
/// for Miri.
///
/// # Memory Requirements (for Multiboot and Rust/Miri)
/// - At least as big as a `size_of<TagHeader>()`
/// - at least [`ALIGNMENT`]-aligned
/// - Length is multiple of [`ALIGNMENT`]. In other words, there are enough
///   padding bytes until so that pointer coming right after the last byte
///   is [`ALIGNMENT`]-aligned
#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct TagBytesRef<'a>(&'a [u8]);

impl<'a> TryFrom<&'a [u8]> for TagBytesRef<'a> {
    type Error = MemoryError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        if value.len() < mem::size_of::<TagHeader>() {
            return Err(MemoryError::MinLengthNotSatisfied);
        }
        // Doesn't work as expected: if align_of_val(&value[0]) < ALIGNMENT {
        if value.as_ptr().align_offset(ALIGNMENT) != 0 {
            return Err(MemoryError::WrongAlignment);
        }
        let padding_bytes = value.len() % ALIGNMENT;
        if padding_bytes != 0 {
            return Err(MemoryError::MissingPadding);
        }
        Ok(Self(value))
    }
}

impl<'a> Deref for TagBytesRef<'a> {
    type Target = &'a [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Errors that occur when constructing a [`TagBytesRef`].
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum MemoryError {
    /// The memory must be at least [`ALIGNMENT`]-aligned.
    WrongAlignment,
    /// The memory must cover at least the length of a [`TagHeader`].
    MinLengthNotSatisfied,
    /// The buffer misses the terminating padding to the next alignment
    /// boundary.
    // This is mainly relevant to satisfy Miri. As the spec also mandates an
    // alignment, we can rely on this property.
    MissingPadding,
}

/// A generic tag serving as base to cast to specific tags. This is a DST
/// version of [`TagHeader`] that solves various type and memory safety
/// problems by having a type that owns the whole memory of a tag.
#[derive(Eq, Ord, PartialEq, PartialOrd, ptr_meta::Pointee)]
#[repr(C)]
pub struct GenericTag {
    header: TagHeader,
    /// Payload of the tag that is reflected in the `size` attribute, thus, no
    /// padding bytes!
    payload: [u8],
}

impl GenericTag {
    /// Base size of the DST struct without the dynamic part.
    const BASE_SIZE: usize = mem::size_of::<TagHeader>();

    /// Creates a reference to a [`GenericTag`] from the provided `bytes`
    /// [`TagBytesRef`].
    pub(crate) fn ref_from(bytes: TagBytesRef) -> &Self {
        let header = bytes.as_ptr().cast::<TagHeader>();
        let header = unsafe { &*header };
        let dst_len = Self::dst_len(header);
        assert_eq!(header.size as usize, Self::BASE_SIZE + dst_len);

        let generic_tag: *const Self = ptr_meta::from_raw_parts(bytes.as_ptr().cast(), dst_len);
        unsafe { &*generic_tag }
    }

    pub const fn header(&self) -> &TagHeader {
        &self.header
    }

    #[cfg(all(test, feature = "builder"))]
    pub const fn payload(&self) -> &[u8] {
        &self.payload
    }

    /// Casts the generic tag to a specific [`TagTrait`] implementation which
    /// may be a ZST or DST typed tag.
    pub fn cast<T: TagTrait + ?Sized>(&self) -> &T {
        let base_ptr = ptr::addr_of!(*self);
        let t_dst_size = T::dst_len(&self.header);
        let t_ptr = ptr_meta::from_raw_parts(base_ptr.cast(), t_dst_size);
        let t_ref = unsafe { &*t_ptr };
        assert_eq!(mem::size_of_val(self), mem::size_of_val(t_ref));
        t_ref
    }
}

impl Debug for GenericTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("GenericTag")
            .field("header", &self.header)
            .field("payload", &"<bytes>")
            .finish()
    }
}

impl TagTrait for GenericTag {
    const ID: TagType = TagType::Custom(0xffffffff);

    fn dst_len(header: &TagHeader) -> usize {
        assert!(header.size as usize >= Self::BASE_SIZE);
        header.size as usize - Self::BASE_SIZE
    }
}

/// Iterates the tags of the MBI from the first tag to the end tag. THe end tag
/// included.
#[derive(Clone, Debug)]
pub struct TagIter<'a> {
    /// Absolute offset to next tag and updated in each iteration.
    next_tag_offset: usize,
    exclusive_end: *const u8,
    buffer: &'a [u8],
}

impl<'a> TagIter<'a> {
    /// Creates a new iterator
    pub fn new(mem: &'a [u8]) -> Self {
        // Assert alignment.
        assert_eq!(mem.as_ptr().align_offset(8), 0);

        let exclusive_end = unsafe { mem.as_ptr().add(mem.len()) };

        TagIter {
            next_tag_offset: 0,
            buffer: mem,
            exclusive_end,
        }
    }
}

impl<'a> Iterator for TagIter<'a> {
    type Item = &'a GenericTag;

    fn next(&mut self) -> Option<Self::Item> {
        let next_ptr = unsafe { self.buffer.as_ptr().add(self.next_tag_offset) };

        if next_ptr == self.exclusive_end {
            return None;
        }
        assert!(next_ptr < self.exclusive_end);

        let next_tag_ptr = next_ptr.cast::<TagHeader>();

        let tag_hdr = unsafe { &*next_tag_ptr };

        // Get relevant byte portion for the next tag. This includes padding
        // bytes to fulfill Rust memory guarantees. Otherwise, Miri complains.
        // See <https://doc.rust-lang.org/reference/type-layout.html>.
        let bytes = {
            let from = self.next_tag_offset;
            let to = from + tag_hdr.size as usize;
            // The size of [the allocation for] a value is always a multiple of its
            // alignment.
            // https://doc.rust-lang.org/reference/type-layout.html
            let to = increase_to_alignment(to);

            // Update ptr for next iteration.
            self.next_tag_offset += to - from;

            &self.buffer[from..to]
        };

        // Should never fail at this point.
        let tag_bytes = TagBytesRef::try_from(bytes).unwrap();

        Some(GenericTag::ref_from(tag_bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::AlignedBytes;
    use core::mem;

    #[test]
    fn test_new_generic_tag() {
        let bytes = AlignedBytes::new([
            /* id: 0xffff_ffff */
            0xff_u8, 0xff_u8, 0xff_u8, 0xff_u8, /* id: 16 */
            16, 0, 0, 0, /* field a: 0xdead_beef */
            0xde, 0xad, 0xbe, 0xef, /* field b: 0x1337_1337 */
            0x13, 0x37, 0x13, 0x37,
        ]);

        let bytes = TagBytesRef::try_from(&bytes[..]).unwrap();
        let tag = GenericTag::ref_from(bytes);
        assert_eq!(tag.header.typ, 0xffff_ffff);
        assert_eq!(tag.header.size, 16);
        assert_eq!(tag.payload.len(), 8);
    }

    #[test]
    fn test_cast_generic_tag_to_sized_tag() {
        #[repr(C)]
        struct CustomTag {
            tag_header: TagHeader,
            a: u32,
            b: u32,
        }

        impl TagTrait for CustomTag {
            const ID: TagType = TagType::End;

            fn dst_len(_header: &TagHeader) -> Self::Metadata {}
        }

        let bytes = AlignedBytes([
            /* id: 0xffff_ffff */
            0xff_u8, 0xff_u8, 0xff_u8, 0xff_u8, /* id: 16 */
            16, 0, 0, 0, /* field a: 0xdead_beef */
            0xef, 0xbe, 0xad, 0xde, /* field b: 0x1337_1337 */
            0x37, 0x13, 0x37, 0x13,
        ]);
        let bytes = TagBytesRef::try_from(&bytes[..]).unwrap();
        let tag = GenericTag::ref_from(bytes);
        let custom_tag = tag.cast::<CustomTag>();

        assert_eq!(mem::size_of_val(custom_tag), 16);
        assert_eq!(custom_tag.a, 0xdead_beef);
        assert_eq!(custom_tag.b, 0x1337_1337);
    }

    #[test]
    fn test_cast_generic_tag_to_dynamically_sized_tag() {
        #[repr(C)]
        #[derive(ptr_meta::Pointee)]
        struct CustomDstTag {
            tag_header: TagHeader,
            a: u32,
            payload: [u8],
        }

        impl TagTrait for CustomDstTag {
            const ID: TagType = TagType::End;

            fn dst_len(header: &TagHeader) -> Self::Metadata {
                let base_size = mem::size_of::<TagHeader>() + mem::size_of::<u32>();
                header.size as usize - base_size
            }
        }

        let bytes = AlignedBytes([
            /* id: 0xffff_ffff */
            0xff_u8, 0xff_u8, 0xff_u8, 0xff_u8, /* id: 16 */
            16, 0, 0, 0, /* field a: 0xdead_beef */
            0xef, 0xbe, 0xad, 0xde, /* field b: 0x1337_1337 */
            0x37, 0x13, 0x37, 0x13,
        ]);

        let bytes = TagBytesRef::try_from(&bytes[..]).unwrap();
        let tag = GenericTag::ref_from(bytes);
        let custom_tag = tag.cast::<CustomDstTag>();

        assert_eq!(mem::size_of_val(custom_tag), 16);
        assert_eq!(custom_tag.a, 0xdead_beef);
        assert_eq!(custom_tag.payload.len(), 4);
        assert_eq!(custom_tag.payload[0], 0x37);
        assert_eq!(custom_tag.payload[1], 0x13);
        assert_eq!(custom_tag.payload[2], 0x37);
        assert_eq!(custom_tag.payload[3], 0x13);
    }

    #[test]
    fn test_tag_bytes_ref() {
        let empty: &[u8] = &[];
        assert_eq!(
            TagBytesRef::try_from(empty),
            Err(MemoryError::MinLengthNotSatisfied)
        );

        let slice = &[0_u8, 1, 2, 3, 4, 5, 6];
        assert_eq!(
            TagBytesRef::try_from(&slice[..]),
            Err(MemoryError::MinLengthNotSatisfied)
        );

        let slice = AlignedBytes([0_u8, 1, 2, 3, 4, 5, 6, 7, 0, 0, 0]);
        // Guaranteed wrong alignment
        let unaligned_slice = &slice[3..];
        assert_eq!(
            TagBytesRef::try_from(unaligned_slice),
            Err(MemoryError::WrongAlignment)
        );

        let slice = AlignedBytes([0_u8, 1, 2, 3, 4, 5, 6, 7]);
        let slice = &slice[..];
        assert_eq!(TagBytesRef::try_from(slice), Ok(TagBytesRef(slice)));
    }

    #[test]
    fn test_create_generic_tag() {
        #[rustfmt::skip]
        let bytes = AlignedBytes::new(
            [
                TagType::Cmdline.val() as u8, 0, 0, 0,
                /* Tag size */
                18, 0, 0, 0,
                /* Some payload.  */
                0, 1, 2, 3,
                4, 5, 6, 7,
                8, 9,
                // Padding
                0, 0, 0, 0, 0, 0
            ],
        );
        let bytes = TagBytesRef::try_from(&bytes[..]).unwrap();
        let tag = GenericTag::ref_from(bytes);
        assert_eq!(tag.header.typ, TagType::Cmdline);
        assert_eq!(tag.header.size, 8 + 10);
    }

    #[test]
    fn test_cast_generic_tag_to_generic_tag() {
        #[rustfmt::skip]
        let bytes = AlignedBytes::new(
            [
                TagType::Cmdline.val() as u8, 0, 0, 0,
                /* Tag size */
                18, 0, 0, 0,
                /* Some payload.  */
                0, 1, 2, 3,
                4, 5, 6, 7,
                8, 9,
                // Padding
                0, 0, 0, 0, 0, 0
            ],
        );
        let bytes = TagBytesRef::try_from(&bytes[..]).unwrap();
        let tag = GenericTag::ref_from(bytes);

        // Main objective here is also that this test passes Miri.
        let tag = tag.cast::<GenericTag>();
        assert_eq!(tag.header.typ, TagType::Cmdline);
        assert_eq!(tag.header.size, 8 + 10);
    }

    #[test]
    fn test_tag_iter() {
        #[rustfmt::skip]
        let bytes = AlignedBytes::new(
            [
                /* Some minimal tag.  */
                0xff, 0, 0, 0,
                8, 0, 0, 0,
                /* Some tag with payload.  */
                0xfe, 0, 0, 0,
                12, 0, 0, 0,
                1, 2, 3, 4,
                // Padding
                0, 0, 0, 0,
                /* End tag */
                0, 0, 0, 0,
                8, 0, 0, 0,
            ],
        );
        let mut iter = TagIter::new(&bytes[..]);
        let first = iter.next().unwrap();
        assert_eq!(first.header.typ, TagType::Custom(0xff));
        assert_eq!(first.header.size, 8);
        assert!(first.payload.is_empty());

        let second = iter.next().unwrap();
        assert_eq!(second.header.typ, TagType::Custom(0xfe));
        assert_eq!(second.header.size, 12);
        assert_eq!(&second.payload, &[1, 2, 3, 4]);

        let third = iter.next().unwrap();
        assert_eq!(third.header.typ, TagType::End);
        assert_eq!(third.header.size, 8);
        assert!(first.payload.is_empty());

        assert_eq!(iter.next(), None);
    }
}
