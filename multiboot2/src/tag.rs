//! Module for the base tag definitions and helper types.
//!
//! The relevant exports of this module is [`Tag`].

use crate::{TagTrait, TagType, TagTypeId};
use core::fmt;
use core::fmt::{Debug, Display, Formatter};
use core::marker::PhantomData;
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
            StringError::MissingNul(e) => Some(e),
            StringError::Utf8(e) => Some(e),
        }
    }
}

/// Common base structure for all tags that can be passed via the Multiboot2
/// Information Structure (MBI) to a Multiboot2 payload/program/kernel.
///
/// Can be transformed to any other tag (sized or unsized/DST) via
/// [`Tag::cast_tag`].
///
/// Do not confuse them with the Multiboot2 header tags. They are something
/// different.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Tag {
    pub typ: TagTypeId, // u32
    pub size: u32,
    // followed by additional, tag specific fields
}

impl Tag {
    /// Returns the underlying type of the tag.
    pub fn typ(&self) -> TagType {
        self.typ.into()
    }

    /// Casts the base tag to the specific tag type.
    pub fn cast_tag<'a, T: TagTrait + ?Sized + 'a>(&'a self) -> &'a T {
        assert_eq!(self.typ, T::ID);
        // Safety: At this point, we trust that "self.size" and the size hint
        // for DST tags are sane.
        unsafe { TagTrait::from_base_tag(self) }
    }

    /// Casts the base tag to the specific tag type, but mutably.
    pub fn cast_tag_mut<'a, T: TagTrait + ?Sized + 'a>(&'a mut self) -> &'a mut T {
        assert_eq!(self.typ, T::ID);
        // Safety: At this point, we trust that "self.size" and the size hint
        // for DST tags are sane.
        unsafe { TagTrait::from_base_tag_mut(self) }
    }

    /// Parses the provided byte sequence as Multiboot string, which maps to a
    /// [`str`].
    pub fn parse_slice_as_string(bytes: &[u8]) -> Result<&str, StringError> {
        let cstr = core::ffi::CStr::from_bytes_until_nul(bytes).map_err(StringError::MissingNul)?;

        cstr.to_str().map_err(StringError::Utf8)
    }
}

impl Debug for Tag {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let tag_type = TagType::from(self.typ);

        let mut debug = f.debug_struct("Tag");
        debug.field("typ", &tag_type);

        if !matches!(tag_type, TagType::Custom(_)) {
            debug.field("typ (numeric)", &(u32::from(self.typ)));
        }

        debug.field("size", &(self.size));

        debug.finish()
    }
}

/// Iterates the MBI's tags from the first tag to the end tag.
#[derive(Clone, Debug)]
pub struct TagIter<'a> {
    /// Pointer to the next tag. Updated in each iteration.
    pub current: *const Tag,
    /// The pointer right after the MBI. Used for additional bounds checking.
    end_ptr_exclusive: *const u8,
    /// Lifetime capture of the MBI's memory.
    _mem: PhantomData<&'a ()>,
}

impl<'a> TagIter<'a> {
    /// Creates a new iterator
    pub fn new(mem: &'a [u8]) -> Self {
        assert_eq!(mem.as_ptr().align_offset(8), 0);
        TagIter {
            current: mem.as_ptr().cast(),
            end_ptr_exclusive: unsafe { mem.as_ptr().add(mem.len()) },
            _mem: PhantomData,
        }
    }
}

impl<'a> Iterator for TagIter<'a> {
    type Item = &'a Tag;

    fn next(&mut self) -> Option<&'a Tag> {
        // This never failed so far. But better be safe.
        assert!(self.current.cast::<u8>() < self.end_ptr_exclusive);

        let tag = unsafe { &*self.current };
        match tag.typ() {
            TagType::End => None, // end tag
            _ => {
                // We return the tag and update self.current already to the next
                // tag.

                // next pointer (rounded up to 8-byte alignment)
                let ptr_offset = (tag.size as usize + 7) & !7;

                // go to next tag
                self.current = unsafe { self.current.cast::<u8>().add(ptr_offset).cast::<Tag>() };

                Some(tag)
            }
        }
    }
}

/// Iterates the MBI's tags from the first tag to the end tag.
#[derive(Clone, Debug)]
pub struct TagIterMut<'a> {
    /// Pointer to the next tag. Updated in each iteration.
    pub current: *mut Tag,
    /// The pointer right after the MBI. Used for additional bounds checking.
    end_ptr_exclusive: *mut u8,
    /// Lifetime capture of the MBI's memory.
    _mem: PhantomData<&'a mut ()>,
}

impl<'a> TagIterMut<'a> {
    /// Creates a new iterator
    pub fn new(mem: &'a mut [u8]) -> Self {
        assert_eq!(mem.as_ptr().align_offset(8), 0);
        TagIterMut {
            current: mem.as_mut_ptr().cast(),
            end_ptr_exclusive: unsafe { mem.as_mut_ptr().add(mem.len()) },
            _mem: PhantomData,
        }
    }
}

impl<'a> Iterator for TagIterMut<'a> {
    type Item = &'a mut Tag;

    fn next(&mut self) -> Option<&'a mut Tag> {
        // This never failed so far. But better be safe.
        assert!(self.current.cast::<u8>() < self.end_ptr_exclusive);

        let tag = unsafe { &mut *self.current };
        match tag.typ() {
            TagType::End => None, // end tag
            _ => {
                // We return the tag and update self.current already to the next
                // tag.

                // next pointer (rounded up to 8-byte alignment)
                let ptr_offset = (tag.size as usize + 7) & !7;

                // go to next tag
                self.current = unsafe { self.current.cast::<u8>().add(ptr_offset).cast::<Tag>() };

                Some(tag)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_slice_as_string() {
        // empty slice is invalid
        assert!(matches!(
            Tag::parse_slice_as_string(&[]),
            Err(StringError::MissingNul(_))
        ));
        // empty string is fine
        assert_eq!(Tag::parse_slice_as_string(&[0x00]), Ok(""));
        // reject invalid utf8
        assert!(matches!(
            Tag::parse_slice_as_string(&[0xff, 0x00]),
            Err(StringError::Utf8(_))
        ));
        // reject missing null
        assert!(matches!(
            Tag::parse_slice_as_string(b"hello"),
            Err(StringError::MissingNul(_))
        ));
        // must not include final null
        assert_eq!(Tag::parse_slice_as_string(b"hello\0"), Ok("hello"));
        assert_eq!(Tag::parse_slice_as_string(b"hello\0\0"), Ok("hello"));
        // must skip everytihng after first null
        assert_eq!(Tag::parse_slice_as_string(b"hello\0foo"), Ok("hello"));
    }
}
