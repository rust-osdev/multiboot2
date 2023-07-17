//! Module for the base tag definition.
//!
//! The relevant exports of this module is [`Tag`].

use crate::{TagTrait, TagType, TagTypeId};
use core::fmt;
use core::fmt::{Debug, Formatter};
use core::marker::PhantomData;
use core::str::Utf8Error;

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

    /// Some multiboot2 tags are a DST as they end with a dynamically sized byte
    /// slice. This function parses this slice as [`str`] so that either a valid
    /// UTF-8 Rust string slice without a terminating null byte or an error is
    /// returned.
    pub fn get_dst_str_slice(bytes: &[u8]) -> Result<&str, Utf8Error> {
        // Determine length of string before first NUL character

        let length = match bytes.iter().position(|ch| (*ch) == 0x00) {
            // Unterminated string case:
            None => bytes.len(),

            // Terminated case:
            Some(idx) => idx,
        };
        // Convert to Rust string:
        core::str::from_utf8(&bytes[..length])
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_dst_str_slice() {
        // unlikely case
        assert_eq!(Ok(""), Tag::get_dst_str_slice(&[]));
        // also unlikely case
        assert_eq!(Ok(""), Tag::get_dst_str_slice(&[b'\0']));
        // unlikely case: missing null byte. but the lib can cope with that
        assert_eq!(Ok("foobar"), Tag::get_dst_str_slice(b"foobar"));
        // test that the null bytes is not included in the string slice
        assert_eq!(Ok("foobar"), Tag::get_dst_str_slice(b"foobar\0"));
        // test that C-style null string termination works as expected
        assert_eq!(Ok("foo"), Tag::get_dst_str_slice(b"foo\0bar"));
        // test invalid utf8
        assert!(matches!(
            Tag::get_dst_str_slice(&[0xff, 0xff]),
            Err(Utf8Error { .. })
        ));
    }
}
