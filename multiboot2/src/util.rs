//! Various utilities.

use crate::ALIGNMENT;
use core::fmt;
use core::fmt::{Display, Formatter};
use core::str::Utf8Error;
#[cfg(feature = "alloc")]
use {
    crate::{TagHeader, TagTrait},
    core::{mem, ptr},
};
#[cfg(feature = "builder")]
use {alloc::alloc::Layout, alloc::boxed::Box};

/// Error type describing failures when parsing the string from a tag.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum StringError {
    /// There is no terminating NUL character, although the specification
    /// requires one.
    MissingNul(core::ffi::FromBytesUntilNulError),
    /// The sequence until the first NUL character is not valid UTF-8.
    Utf8(Utf8Error),
}

impl Display for StringError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(feature = "unstable")]
impl core::error::Error for StringError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::MissingNul(e) => Some(e),
            Self::Utf8(e) => Some(e),
        }
    }
}

/// Creates a new tag implementing [`TagTrait`] on the heap. This works for
/// sized and unsized tags. However, it only makes sense to use this for tags
/// that are DSTs (unsized), as for the sized ones, you can call a regular
/// constructor and box the result.
///
/// # Parameters
/// - `additional_bytes_slices`: Array of byte slices that should be included
///   without additional padding in-between. You don't need to add the bytes
///   for [`TagHeader`], but only additional ones.
#[cfg(feature = "alloc")]
#[must_use]
pub fn new_boxed<T: TagTrait + ?Sized>(additional_bytes_slices: &[&[u8]]) -> Box<T> {
    let additional_size = additional_bytes_slices
        .iter()
        .map(|b| b.len())
        .sum::<usize>();

    let size = mem::size_of::<TagHeader>() + additional_size;
    let alloc_size = increase_to_alignment(size);
    let layout = Layout::from_size_align(alloc_size, ALIGNMENT).unwrap();
    let heap_ptr = unsafe { alloc::alloc::alloc(layout) };
    assert!(!heap_ptr.is_null());

    unsafe {
        heap_ptr.cast::<u32>().write(T::ID.val());
        heap_ptr.cast::<u32>().add(1).write(size as u32);
    }

    let mut write_offset = mem::size_of::<TagHeader>();
    for &bytes in additional_bytes_slices {
        unsafe {
            let len = bytes.len();
            let src = bytes.as_ptr();
            let dst = heap_ptr.add(write_offset);
            ptr::copy_nonoverlapping(src, dst, len);
            write_offset += len;
        }
    }

    let header = unsafe { heap_ptr.cast::<TagHeader>().as_ref() }.unwrap();

    let ptr = ptr_meta::from_raw_parts_mut(heap_ptr.cast(), T::dst_len(header));

    unsafe { Box::from_raw(ptr) }
}

/// Parses the provided byte sequence as Multiboot string, which maps to a
/// [`str`].
pub fn parse_slice_as_string(bytes: &[u8]) -> Result<&str, StringError> {
    let cstr = core::ffi::CStr::from_bytes_until_nul(bytes).map_err(StringError::MissingNul)?;
    cstr.to_str().map_err(StringError::Utf8)
}

/// Increases the given size to the next alignment boundary, if it is not a
/// multiple of the alignment yet. This is relevant as in Rust's [type layout],
/// the allocated size of a type is always a multiple of the alignment, even
/// if the type is smaller.
///
/// [type layout]: https://doc.rust-lang.org/reference/type-layout.html
pub const fn increase_to_alignment(size: usize) -> usize {
    let mask = ALIGNMENT - 1;
    (size + mask) & !mask
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "alloc")]
    use {crate::tag::GenericTag, crate::CommandLineTag};

    #[test]
    fn test_parse_slice_as_string() {
        // empty slice is invalid
        assert!(matches!(
            parse_slice_as_string(&[]),
            Err(StringError::MissingNul(_))
        ));
        // empty string is fine
        assert_eq!(parse_slice_as_string(&[0x00]), Ok(""));
        // reject invalid utf8
        assert!(matches!(
            parse_slice_as_string(&[0xff, 0x00]),
            Err(StringError::Utf8(_))
        ));
        // reject missing null
        assert!(matches!(
            parse_slice_as_string(b"hello"),
            Err(StringError::MissingNul(_))
        ));
        // must not include final null
        assert_eq!(parse_slice_as_string(b"hello\0"), Ok("hello"));
        assert_eq!(parse_slice_as_string(b"hello\0\0"), Ok("hello"));
        // must skip everytihng after first null
        assert_eq!(parse_slice_as_string(b"hello\0foo"), Ok("hello"));
    }

    #[test]
    fn test_increase_to_alignment() {
        assert_eq!(increase_to_alignment(0), 0);
        assert_eq!(increase_to_alignment(1), 8);
        assert_eq!(increase_to_alignment(7), 8);
        assert_eq!(increase_to_alignment(8), 8);
        assert_eq!(increase_to_alignment(9), 16);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_new_boxed() {
        let tag = new_boxed::<GenericTag>(&[&[0, 1, 2, 3]]);
        assert_eq!(tag.header().typ, GenericTag::ID);
        assert_eq!(tag.payload(), &[0, 1, 2, 3]);

        // Test that bytes are added consecutively without gaps.
        let tag = new_boxed::<GenericTag>(&[&[0], &[1], &[2, 3]]);
        assert_eq!(tag.header().typ, GenericTag::ID);
        assert_eq!(tag.payload(), &[0, 1, 2, 3]);

        let tag = new_boxed::<CommandLineTag>(&[b"hello\0"]);
        assert_eq!(tag.cmdline(), Ok("hello"));
    }
}