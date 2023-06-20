use crate::{HeaderTagFlag, MbiTagType};
use crate::{HeaderTagType, MbiTagTypeId};
use core::fmt;
use core::fmt::{Debug, Formatter};
use core::marker::PhantomData;
use core::mem::size_of;
use multiboot2::TagType;

/// Specifies what specific tag types the bootloader should provide
/// inside the mbi.
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct InformationRequestHeaderTag<const N: usize> {
    typ: HeaderTagType,
    flags: HeaderTagFlag,
    size: u32,
    // Length is determined by size.
    // Must be parsed during runtime with unsafe pointer magic and the size field.
    requests: [MbiTagTypeId; N],
}

impl<const N: usize> InformationRequestHeaderTag<N> {
    /// Creates a new object. The size parameter is the value of the size property.
    /// It doesn't have to match with `N` necessarily, because during compile time we
    /// can't know the size of the tag in all runtime situations.
    pub fn new(flags: HeaderTagFlag, requests: [MbiTagTypeId; N], size: Option<u32>) -> Self {
        InformationRequestHeaderTag {
            typ: HeaderTagType::InformationRequest,
            flags,
            size: size.unwrap_or(size_of::<Self>() as u32),
            requests,
        }
    }

    pub const fn typ(&self) -> HeaderTagType {
        self.typ
    }
    pub const fn flags(&self) -> HeaderTagFlag {
        self.flags
    }
    pub const fn size(&self) -> u32 {
        self.size
    }

    /// Returns the requests as array. Only works if the number of requests
    /// is known at compile time. For safety and correctness during runtime,
    /// you should use `req_iter()`.
    pub const fn requests(&self) -> [MbiTagTypeId; N] {
        // cheap to copy, otherwise difficult with lifetime
        self.requests
    }

    /// Returns the number of [`MbiTagType`]-requests derived
    /// from the `size`-property. This method is useful
    /// because this struct uses a const generic, but during runtime
    /// we don't know the value in almost any case.
    pub const fn dynamic_requests_size(&self) -> u32 {
        let base_struct_size = size_of::<InformationRequestHeaderTag<0>>();
        let size_diff = self.size - base_struct_size as u32;
        if size_diff > 0 {
            size_diff / size_of::<u32>() as u32
        } else {
            0
        }
    }

    /// Returns an [`InformationRequestHeaderTagIter`].
    pub fn req_iter(&self) -> InformationRequestHeaderTagIter {
        let base_struct_size = size_of::<InformationRequestHeaderTag<0>>();
        let count = self.dynamic_requests_size();
        let base_ptr = self as *const InformationRequestHeaderTag<N>;
        let base_ptr = base_ptr as *const u8;
        let base_ptr = unsafe { base_ptr.add(base_struct_size) };
        let base_ptr = base_ptr as *const MbiTagTypeId;
        InformationRequestHeaderTagIter::new(count, base_ptr)
    }
}

impl<const N: usize> Debug for InformationRequestHeaderTag<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("InformationRequestHeaderTag")
            .field("type", &{ self.typ })
            .field("flags", &{ self.flags })
            .field("size", &{ self.size })
            .field("requests", &{ self.req_iter() })
            .finish()
    }
}

/// Iterates the dynamically sized information request structure and finds all MBI tags
/// that are requested.
#[derive(Copy, Clone)]
pub struct InformationRequestHeaderTagIter<'a> {
    base_ptr: *const MbiTagTypeId,
    i: u32,
    count: u32,
    _marker: PhantomData<&'a ()>,
}

impl<'a> InformationRequestHeaderTagIter<'a> {
    const fn new(count: u32, base_ptr: *const MbiTagTypeId) -> Self {
        Self {
            i: 0,
            count,
            base_ptr,
            _marker: PhantomData,
        }
    }
}

impl<'a> Iterator for InformationRequestHeaderTagIter<'a> {
    type Item = MbiTagType;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i < self.count {
            let ptr = unsafe { self.base_ptr.offset(self.i as isize) };
            self.i += 1;
            let tag_type_id = unsafe { *ptr };
            Some(TagType::from(tag_type_id))
        } else {
            None
        }
    }
}

impl<'a> Debug for InformationRequestHeaderTagIter<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_list();
        self.for_each(|e| {
            debug.entry(&e);
        });
        debug.finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::InformationRequestHeaderTag;

    #[test]
    #[allow(clippy::erasing_op)]
    #[allow(clippy::identity_op)]
    fn test_assert_size() {
        assert_eq!(
            core::mem::size_of::<InformationRequestHeaderTag<0>>(),
            2 + 2 + 4 + 0 * 4
        );
        assert_eq!(
            core::mem::size_of::<InformationRequestHeaderTag<1>>(),
            2 + 2 + 4 + 1 * 4
        );
        assert_eq!(
            core::mem::size_of::<InformationRequestHeaderTag<2>>(),
            2 + 2 + 4 + 2 * 4
        );
    }
}
