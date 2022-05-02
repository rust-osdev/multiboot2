use crate::HeaderTagType;
#[cfg(feature = "builder")]
use crate::StructAsBytes;
use crate::{HeaderTagFlag, MbiTagType};
#[cfg(feature = "builder")]
use alloc::collections::BTreeSet;
#[cfg(feature = "builder")]
use alloc::vec::Vec;
use core::fmt;
use core::fmt::{Debug, Formatter};
use core::marker::PhantomData;
use core::mem::size_of;

/// Specifies what specific tag types the bootloader should provide
/// inside the mbi.
#[derive(Copy, Clone)]
#[repr(C, packed(8))]
pub struct InformationRequestHeaderTag<const N: usize> {
    typ: HeaderTagType,
    flags: HeaderTagFlag,
    size: u32,
    // Length is determined by size.
    // Must be parsed during runtime with unsafe pointer magic and the size field.
    requests: [MbiTagType; N],
}

impl<const N: usize> InformationRequestHeaderTag<N> {
    /// Creates a new object. The size parameter is the value of the size property.
    /// It doesn't have to match with `N` necessarily, because during compile time we
    /// can't know the size of the tag in all runtime situations.
    pub fn new(flags: HeaderTagFlag, requests: [MbiTagType; N], size: Option<u32>) -> Self {
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
    pub const fn requests(&self) -> [MbiTagType; N] {
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
        let base_ptr = base_ptr as *const MbiTagType;
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

#[cfg(feature = "builder")]
impl<const N: usize> crate::StructAsBytes for InformationRequestHeaderTag<N> {}

/// Helper to build the dynamically sized [`InformationRequestHeaderTag`]
/// at runtime.
#[derive(Debug)]
#[cfg(feature = "builder")]
pub struct InformationRequestHeaderTagBuilder {
    flag: HeaderTagFlag,
    irs: BTreeSet<MbiTagType>,
}

#[cfg(feature = "builder")]
impl InformationRequestHeaderTagBuilder {
    /// New builder.
    pub fn new(flag: HeaderTagFlag) -> Self {
        Self {
            irs: BTreeSet::new(),
            flag,
        }
    }

    /// Returns the expected length of the information request tag,
    /// when the `build`-method gets called.
    pub fn expected_len(&self) -> usize {
        let basic_header_size = size_of::<InformationRequestHeaderTag<0>>();
        let req_tags_size = self.irs.len() * size_of::<MbiTagType>();
        basic_header_size + req_tags_size
    }

    /// Adds an [`MbiTagType`] to the information request.
    pub fn add_ir(mut self, tag: MbiTagType) -> Self {
        self.irs.insert(tag);
        self
    }

    /// Adds multiple [`MbiTagType`] to the information request.
    pub fn add_irs(mut self, tags: &[MbiTagType]) -> Self {
        self.irs.extend(tags);
        self
    }

    /// Builds the bytes of the dynamically sized information request header.
    pub fn build(self) -> Vec<u8> {
        let expected_len = self.expected_len();
        let mut data = Vec::with_capacity(expected_len);

        let basic_tag = InformationRequestHeaderTag::<0>::new(
            self.flag,
            [],
            // we put the expected length here already, because in the next step we write
            // all the tags into the byte array. We can't know this during compile time,
            // therefore N is 0.
            Some(expected_len as u32),
        );
        data.extend(basic_tag.struct_as_bytes());
        #[cfg(debug_assertions)]
        {
            let basic_tag_size = size_of::<InformationRequestHeaderTag<0>>();
            assert_eq!(
                data.len(),
                basic_tag_size,
                "the vector must be as long as the basic tag!"
            );
        }

        for tag in &self.irs {
            let bytes: [u8; 4] = (*tag as u32).to_ne_bytes();
            data.extend(&bytes);
        }

        debug_assert_eq!(
            data.len(),
            expected_len,
            "the byte vector must be as long as the expected size of the struct"
        );

        data
    }
}

/// Iterates the dynamically sized information request structure and finds all MBI tags
/// that are requested.
#[derive(Copy, Clone)]
pub struct InformationRequestHeaderTagIter<'a> {
    base_ptr: *const MbiTagType,
    i: u32,
    count: u32,
    _marker: PhantomData<&'a ()>,
}

impl<'a> InformationRequestHeaderTagIter<'a> {
    fn new(count: u32, base_ptr: *const MbiTagType) -> Self {
        Self {
            i: 0,
            count,
            base_ptr,
            _marker: PhantomData::default(),
        }
    }
}

impl<'a> Iterator for InformationRequestHeaderTagIter<'a> {
    type Item = &'a MbiTagType;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i < self.count {
            let ptr = unsafe { self.base_ptr.offset(self.i as isize) };
            self.i += 1;
            Some(unsafe { &*ptr })
        } else {
            None
        }
    }
}

impl<'a> Debug for InformationRequestHeaderTagIter<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_list();
        (*self).for_each(|e| {
            debug.entry(e);
        });
        debug.finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        HeaderTagFlag, InformationRequestHeaderTag, InformationRequestHeaderTagBuilder, MbiTagType,
    };

    #[test]
    fn test_builder() {
        let builder = InformationRequestHeaderTagBuilder::new(HeaderTagFlag::Required)
            .add_ir(MbiTagType::EfiMmap)
            .add_ir(MbiTagType::BootLoaderName)
            .add_ir(MbiTagType::Cmdline);
        // type(u16) + flags(u16) + size(u32) + 3 tags (u32)
        assert_eq!(builder.expected_len(), 2 + 2 + 4 + 3 * 4);
        let tag = builder.build();
        let tag = unsafe {
            (tag.as_ptr() as *const InformationRequestHeaderTag<3>)
                .as_ref()
                .unwrap()
        };
        assert_eq!(tag.flags, HeaderTagFlag::Required);
        // type(u16) + flags(u16) + size(u32) + 3 tags (u32)
        assert_eq!(tag.size, 2 + 2 + 4 + 3 * 4);
        assert_eq!(tag.dynamic_requests_size(), 3);
        assert!(tag.requests.contains(&MbiTagType::EfiMmap));
        assert!(tag.requests.contains(&MbiTagType::BootLoaderName));
        assert!(tag.requests.contains(&MbiTagType::Cmdline));
        assert_eq!(tag.requests.len(), 3);
        assert!(!tag.requests.contains(&MbiTagType::AcpiV1));
        println!("{:#?}", tag);
    }
}
