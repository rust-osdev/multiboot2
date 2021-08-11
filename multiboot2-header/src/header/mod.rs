pub mod builder;

pub use self::builder::*;
use crate::{
    AddressHeaderTag, InformationRequestHeaderTag, RelocatableHeaderTag, StructAsBytes,
    MULTIBOOT2_HEADER_MAGIC,
};
use crate::{ConsoleHeaderTag, EntryHeaderTag};
use crate::{EfiBootServiceHeaderTag, FramebufferHeaderTag};
use crate::{EndHeaderTag, HeaderTagType};
use crate::{EntryEfi32HeaderTag, EntryEfi64HeaderTag};
use crate::{HeaderTag, HeaderTagISA};
use core::fmt::{Debug, Formatter};
use core::mem::size_of;

/// Wrapper type around a pointer to the Multiboot2 header.
/// Use this if you get a pointer to the header and just want
/// to parse it. If you want to construct the type by yourself,
/// please look at [`Multiboot2HeaderInner`].
#[repr(transparent)]
pub struct Multiboot2Header<'a> {
    inner: &'a Multiboot2HeaderInner,
}

impl<'a> Multiboot2Header<'a> {
    /// Creates a new [`Multiboot2Header`]. Verifies the checksum.
    pub fn new(inner: &'a Multiboot2HeaderInner) -> Self {
        assert!(inner.verify_checksum(), "checksum invalid!");
        Self { inner }
    }

    /*
      Nope, this just results in really ugly code :/
      I didn't find a nice way to solve this.

    /// * `additional_tag_length`: additional tag length including end tag
    pub fn new(arch: ISA,
               additional_tag_length: u32) -> (Multiboot2HeaderInner, Self) {
        let inner = Multiboot2HeaderInner::new(
            arch,
            (size_of::<Multiboot2HeaderInner>() + additional_tag_length as usize) as u32
        );
        let inner_ref = &inner;
        (inner, Self{inner: inner_ref,})
    }*/

    /// Returns the size a static multiboot2 header without tags has.
    pub fn static_size() -> u32 {
        size_of::<Multiboot2HeaderInner>() as u32
    }

    pub fn verify_checksum(&self) -> bool {
        self.inner.verify_checksum()
    }

    pub fn header_magic(&self) -> u32 {
        self.inner.header_magic()
    }
    pub fn arch(&self) -> HeaderTagISA {
        self.inner.arch()
    }
    pub fn length(&self) -> u32 {
        self.inner.length()
    }
    pub fn checksum(&self) -> u32 {
        self.inner.checksum()
    }
    pub fn iter(&self) -> Multiboot2HeaderTagIter {
        self.inner.tag_iter()
    }

    pub fn calc_checksum(magic: u32, arch: HeaderTagISA, length: u32) -> u32 {
        (0x100000000 - magic as u64 - arch as u64 - length as u64) as u32
    }
}

impl<'a> Debug for Multiboot2Header<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        // For debug fmt we only output the inner field
        let reference = unsafe { &*(self.inner as *const Multiboot2HeaderInner) };
        Debug::fmt(reference, f)
    }
}

/// The actual multiboot2 header. The size is not known at
/// compile time and can only be derived during runtime from
/// the iteration of the tags until the end tag.
///
/// Use this only if you know what you do. You probably want to use
/// [`Multiboot2Header`]. If you want to construct a Multiboot2 header at
/// runtime, see [`builder::Multiboot2HeaderBuilder`].
#[derive(Copy, Clone)]
#[repr(C, packed(8))]
pub struct Multiboot2HeaderInner {
    /// Must be the value of [`MULTIBOOT2_HEADER_MAGIC`].
    header_magic: u32,
    arch: HeaderTagISA,
    length: u32,
    checksum: u32,
    // additional tags..
    // at minimum the end tag
}

impl Multiboot2HeaderInner {
    /// Constructs a new
    pub const fn new(arch: HeaderTagISA, length: u32) -> Self {
        let magic = MULTIBOOT2_HEADER_MAGIC;
        let checksum = Self::calc_checksum(magic, arch, length);
        Multiboot2HeaderInner {
            header_magic: magic,
            arch,
            length,
            checksum,
        }
    }

    /// Verifies that a multiboot2 header is valid
    pub(super) fn verify_checksum(&self) -> bool {
        let check = Self::calc_checksum(self.header_magic, self.arch, self.length);
        check == self.checksum
    }

    const fn calc_checksum(magic: u32, arch: HeaderTagISA, length: u32) -> u32 {
        (0x100000000 - magic as u64 - arch as u64 - length as u64) as u32
    }

    fn header_magic(&self) -> u32 {
        self.header_magic
    }
    fn arch(&self) -> HeaderTagISA {
        self.arch
    }
    fn length(&self) -> u32 {
        self.length
    }
    fn checksum(&self) -> u32 {
        self.checksum
    }

    fn tag_iter(&self) -> Multiboot2HeaderTagIter {
        let base_hdr_size = size_of::<Multiboot2HeaderInner>();
        if base_hdr_size == self.length as usize {
            panic!("No end tag!");
        }
        let tag_base_addr = self as *const Multiboot2HeaderInner;
        // cast to u8 so that the offset in bytes works correctly
        let tag_base_addr = tag_base_addr as *const u8;
        // tag_base_addr should now point behind the "static" members
        let tag_base_addr = unsafe { tag_base_addr.offset(base_hdr_size as isize) };
        // align pointer to 8 byte according to spec
        let tag_base_addr = unsafe { tag_base_addr.offset(tag_base_addr.align_offset(8) as isize) };
        // cast back
        let tag_base_addr = tag_base_addr as *const HeaderTag;
        let tags_len = self.length as usize - base_hdr_size;
        Multiboot2HeaderTagIter::new(tag_base_addr, tags_len as u32)
    }
}

impl Debug for Multiboot2HeaderInner {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Multiboot2Header")
            .field("header_magic", &{ self.header_magic })
            .field("arch", &{ self.arch })
            .field("length", &{ self.length })
            .field("checksum", &{ self.checksum })
            .field("tags", &self.tag_iter())
            .finish()
    }
}

impl StructAsBytes for Multiboot2HeaderInner {}

#[derive(Clone)]
pub struct Multiboot2HeaderTagIter {
    /// 8-byte aligned base address
    base: *const HeaderTag,
    /// Offset in bytes from the base address.
    /// Always <= than size.
    n: u32,
    /// Size / final value of [`Self::n`].
    size: u32,
}

impl Multiboot2HeaderTagIter {
    fn new(base: *const HeaderTag, size: u32) -> Self {
        // transform to byte pointer => offset works properly
        let base = base as *const u8;
        let base = unsafe { base.offset(base.align_offset(8) as isize) };
        let base = base as *const HeaderTag;
        Self { base, n: 0, size }
    }
}

impl Iterator for Multiboot2HeaderTagIter {
    type Item = *const HeaderTag;

    fn next(&mut self) -> Option<Self::Item> {
        if self.n < self.size {
            // transform to byte ptr => offset works correctly
            let ptr = self.base as *const u8;
            let ptr = unsafe { ptr.offset(self.n as isize) };
            let ptr = ptr as *const HeaderTag;
            assert_eq!(ptr as usize % 8, 0, "must be 8-byte aligned");
            let tag = unsafe { &*ptr };
            assert!(
                tag.size() <= 500,
                "no real mb2 header should be bigger than 500bytes - probably wrong memory?! is: {}",
                {tag.size()}
            );
            self.n += tag.size();
            // 8-byte alignment of pointer address
            self.n += self.n % 8;
            Some(ptr)
        } else {
            None
        }
    }
}

impl Debug for Multiboot2HeaderTagIter {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut debug = f.debug_list();
        self.clone().for_each(|t| unsafe {
            let typ = (*t).typ();
            if typ == HeaderTagType::End {
                let entry = t as *const EndHeaderTag;
                let entry = &*(entry);
                debug.entry(entry);
            } else if typ == HeaderTagType::InformationRequest {
                let entry = t as *const InformationRequestHeaderTag<0>;
                let entry = &*(entry);
                debug.entry(entry);
            } else if typ == HeaderTagType::Address {
                let entry = t as *const AddressHeaderTag;
                let entry = &*(entry);
                debug.entry(entry);
            } else if typ == HeaderTagType::EntryAddress {
                let entry = t as *const EntryHeaderTag;
                let entry = &*(entry);
                debug.entry(entry);
            } else if typ == HeaderTagType::ConsoleFlags {
                let entry = t as *const ConsoleHeaderTag;
                let entry = &*(entry);
                debug.entry(entry);
            } else if typ == HeaderTagType::Framebuffer {
                let entry = t as *const FramebufferHeaderTag;
                let entry = &*(entry);
                debug.entry(entry);
            } else if typ == HeaderTagType::EfiBS {
                let entry = t as *const EfiBootServiceHeaderTag;
                let entry = &*(entry);
                debug.entry(entry);
            } else if typ == HeaderTagType::EntryAddressEFI32 {
                let entry = t as *const EntryEfi32HeaderTag;
                let entry = &*(entry);
                debug.entry(entry);
            } else if typ == HeaderTagType::EntryAddressEFI64 {
                let entry = t as *const EntryEfi64HeaderTag;
                let entry = &*(entry);
                debug.entry(entry);
            } else if typ == HeaderTagType::Relocatable {
                let entry = t as *const RelocatableHeaderTag;
                let entry = &*(entry);
                debug.entry(entry);
            } else {
                panic!("unknown tag ({:?})!", typ);
            }
        });
        debug.finish()
    }
}
