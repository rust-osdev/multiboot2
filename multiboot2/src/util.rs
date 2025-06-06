//! Various utilities.

use core::str::Utf8Error;
use thiserror::Error;

/// Error type describing failures when parsing the string from a tag.
#[derive(Debug, PartialEq, Eq, Clone, Error)]
pub enum StringError {
    /// There is no terminating NUL character, although the specification
    /// requires one.
    #[error("string is not null terminated")]
    MissingNul(#[source] core::ffi::FromBytesUntilNulError),
    /// The sequence until the first NUL character is not valid UTF-8.
    #[error("string is not valid UTF-8")]
    Utf8(#[source] Utf8Error),
}

/// Parses the provided byte sequence as Multiboot string, which maps to a
/// [`str`].
pub fn parse_slice_as_string(bytes: &[u8]) -> Result<&str, StringError> {
    let cstr = core::ffi::CStr::from_bytes_until_nul(bytes).map_err(StringError::MissingNul)?;
    cstr.to_str().map_err(StringError::Utf8)
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
        // must skip everything after first null
        assert_eq!(parse_slice_as_string(b"hello\0foo"), Ok("hello"));
    }
}
