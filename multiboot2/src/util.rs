//! Various utilities.

use crate::ALIGNMENT;
use core::fmt;
use core::fmt::{Display, Formatter};
use core::str::Utf8Error;

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
}
