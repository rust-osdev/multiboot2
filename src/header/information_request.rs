use alloc::fmt;
use core::fmt::{Debug, Formatter};
use core::marker::PhantomData;
use core::mem::size_of;
use HeaderTagFlag;
use {HeaderTagType, MbiTagType};

/// Specifies what specific tag types the bootloader should provide
/// inside the mbi.
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct InformationRequestHeaderTag<const N: usize> {
    typ: HeaderTagType,
    flags: HeaderTagFlag,
    size: u32,
    // Length is determined by size.
    // Must be parsed during runtime with unsafe pointer magic and the size field.
    requests: [MbiTagType; N],
}

impl<const N: usize> InformationRequestHeaderTag<N> {
    pub fn new(flags: HeaderTagFlag, requests: [MbiTagType; N]) -> Self {
        InformationRequestHeaderTag {
            typ: HeaderTagType::InformationRequest,
            flags,
            size: size_of::<Self>() as u32,
            requests,
        }
    }

    pub fn typ(&self) -> HeaderTagType {
        self.typ
    }
    pub fn flags(&self) -> HeaderTagFlag {
        self.flags
    }
    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn requests(&self) -> [MbiTagType; N] {
        // cheap to clone, otherwise difficult with lifetime
        { self.requests }.clone()
    }

    pub fn req_iter(&self) -> InformationRequestHeaderTagIter {
        let base_size = size_of::<InformationRequestHeaderTag<0>>() as u32;
        let count = (self.size - base_size) / size_of::<u32>() as u32;
        let base_ptr = self as *const InformationRequestHeaderTag<N>;
        let base_ptr = base_ptr as *const u8;
        let base_ptr = unsafe { base_ptr.offset(base_size as isize) };
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
            // .field("requests", &)
            .field("req_iter", &{ self.req_iter() })
            .finish()
    }
}

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
        self.clone().for_each(|e| {
            debug.entry(e);
        });
        debug.finish()
    }
}
