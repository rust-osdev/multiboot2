//! Type definitions for "multiboot2 header tags". These tags occur in a binary if it
//! is multiboot2-compliant, for example a kernel.
//!
//! The values are taken from the example C code at the end of the official multiboot2 spec.
//!
//! This type definitions are only beneficial to parse such a structure. They can't be used
//! to construct a multiboot2 header as a static global variable. To write a multiboot2
//! header, it is highly recommended to do this directly in assembly, because of the
//! unknown size of structs or some addresses are not known yet (keyword: relocations).

// the names speak for themselves in most cases
#![allow(missing_docs)]

use alloc::vec::Vec;
use core::fmt;
use core::fmt::{Debug, Formatter};
use core::mem::size_of;
use tags::mbi::TagType;
use core::marker::PhantomData;


/// Value must be present in multiboot2 header.
pub const MULTIBOOT2_HEADER_MAGIC: u32 = 0xe85250d6;

pub unsafe fn load_mb2_header<'a>(addr: usize) -> Multiboot2Header<'a> {
    assert_ne!(0, addr, "null pointer");
    assert_eq!(addr % 8, 0, "must be 8-byte aligned, see multiboot spec");
    let ptr = addr as *const Multiboot2HeaderInner;
    assert!((*ptr).verify_checksum(), "checksum invalid!");
    let reference = &*ptr;
    Multiboot2Header { inner: reference }
}

/// This struct is useful to construct multiboot2 headers for tests during runtime.
/// For production, I recommend to add them via assembly into a separate object file.
/// This is usually much simpler.
#[derive(Debug)]
pub struct Multiboot2HeaderBuilder {
    additional_tags_len: Option<u32>,
    arch: Option<HeaderTagISA>,
}

impl Multiboot2HeaderBuilder {
    pub fn new() -> Self {
        Self {
            additional_tags_len: None,
            arch: None,
        }
    }

    pub fn additional_tags_len(mut self, additional_tags_len: u32) -> Self {
        self.additional_tags_len = Some(additional_tags_len);
        self
    }

    pub fn arch(mut self, arch: HeaderTagISA) -> Self {
        self.arch = Some(arch);
        self
    }

    /// Constructs the bytes for the static part of the header only!
    /// Useful to write tests and create multiboot2 headers during runtime.
    pub fn build(self) -> Vec<u8> {
        let arch = self.arch.unwrap();
        let static_length = size_of::<Multiboot2HeaderInner>();
        let length = static_length as u32 + self.additional_tags_len.unwrap_or(0);
        let checksum = Multiboot2HeaderInner::calc_checksum(MULTIBOOT2_HEADER_MAGIC, arch, length);
        let hdr = Multiboot2HeaderInner {
            header_magic: MULTIBOOT2_HEADER_MAGIC,
            arch,
            length,
            checksum,
        };
        let ptr = (&hdr) as *const Multiboot2HeaderInner as *const u8;
        let mut buf = Vec::with_capacity(static_length);
        for i in 0..static_length {
            let byte = unsafe { *ptr.offset(i as isize) };
            buf.push(byte);
        }
        buf
    }
}

#[repr(transparent)]
pub struct Multiboot2Header<'a> {
    inner: &'a Multiboot2HeaderInner,
}

impl<'a> Multiboot2Header<'a> {
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

#[derive(Copy, Clone)]
#[repr(C, packed)]
struct Multiboot2HeaderInner {
    /// Must be the value of [`MULTIBOOT2_HEADER_MAGIC`].
    header_magic: u32,
    arch: HeaderTagISA,
    length: u32,
    checksum: u32,
    // additional tags..
    // at minimum the end tag
}

impl Multiboot2HeaderInner {
    /* Nope, this just results in really ugly code :/
      I didn't find a nice way to solve this.
    fn new(arch: ISA, length: u32) -> Self {
        let magic = MULTIBOOT2_HEADER_MAGIC;
        let checksum = Self::calc_checksum(magic, arch, length);
        Multiboot2HeaderInner { header_magic: magic, arch, length, checksum }
    }*/

    /// Verifies that a multiboot2 header is valid
    fn verify_checksum(&self) -> bool {
        let check = Self::calc_checksum(self.header_magic, self.arch, self.length);
        check == self.checksum
    }

    fn calc_checksum(magic: u32, arch: HeaderTagISA, length: u32) -> u32 {
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
            .field("header_magic", &{self.header_magic})
            .field("arch", &{self.arch})
            .field("length", &{self.length})
            .field("checksum", &{self.checksum})
            .field("tags", &self.tag_iter())
            .finish()
    }
}

#[derive(Clone)]
pub struct Multiboot2HeaderTagIter {
    /// 8-byte aligned base address
    base: *const HeaderTag,
    /// Offset in bytes from the base address.
    /// Always <= than size.
    n: u32,
    /// Size / final value of [`n`].
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
                tag.size <= 500,
                "no real mb2 header should be bigger than 500bytes - probably wrong memory?! is: {}",
                {tag.size}
            );
            self.n += tag.size;
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
            let typ = (*t).typ;
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

/// ISA/ARCH in multiboot2 header.
#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum HeaderTagISA {
    /// Spec: "means 32-bit (protected) mode of i386".
    /// Caution: This is confusing. If you use the EFI64-tag
    /// on an UEFI system, you get into `64-bit long mode`.
    /// Therefore this tag should be understood as "arch=x86".
    I386 = 0,
    /// 32-bit MIPS
    MIPS32 = 4,
}

/// Possible types for header tags of a multiboot2 header. The names and values are taken
/// from the example C code at the bottom of the Multiboot2 specification.
#[repr(u16)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum HeaderTagType {
    /// Type for [`EndHeaderTag`].
    End = 0,
    /// Type for [`InformationRequestHeaderTag`].
    InformationRequest = 1,
    /// Type for [`AddressHeaderTag`].
    Address = 2,
    /// Type for [`EntryHeaderTag`].
    EntryAddress = 3,
    /// Type for [`ConsoleHeaderTag`].
    ConsoleFlags = 4,
    /// Type for [`FramebufferHeaderTag`].
    Framebuffer = 5,
    /// Type for [`ModuleAlignHeaderTag`].
    ModuleAlign = 6,
    /// Type for [`EfiBootServiceHeaderTag`].
    EfiBS = 7,
    /// Type for [`EntryEfi32HeaderTag`].
    EntryAddressEFI32 = 8,
    /// Type for [`EntryEfi64HeaderTag`].
    EntryAddressEFI64 = 9,
    /// Type for [`RelocatableHeaderTag`].
    Relocatable = 10,
}

/// Flags for multiboot2 header tags.
#[repr(u16)]
#[derive(Copy, Clone, Debug)]
pub enum HeaderTagFlag {
    Required = 0,
    Optional = 1,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum ConsoleHeaderTagFlags {
    /// Console required.
    ConsoleRequired = 0,
    /// EGA text support.
    EgaTextSupported = 1,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum RelocatableHeaderTagPreference {
    /// Let boot loader decide.
    None = 0,
    /// Locate at lower end of possible address space.
    Low = 1,
    /// Locate at higher end of possible address space.
    High = 2,
}

/// Common structure for all header tags.
#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct HeaderTag {
    // u16 value
    typ: HeaderTagType,
    // u16 value
    flags: HeaderTagFlag,
    size: u32,
    // maybe additional fields (tag specific)
}

impl HeaderTag {
    // never needed to construct this publicly
    /*
    pub fn new(typ: HeaderTagType, flags: HeaderTagFlag, size: u32) -> Self {
        HeaderTag { typ, flags, size }
    }*/

    pub fn typ(&self) -> HeaderTagType {
        self.typ
    }
    pub fn flags(&self) -> HeaderTagFlag {
        self.flags
    }
    pub fn size(&self) -> u32 {
        self.size
    }
}

/// Terminates a list of optional tags
/// in a multiboot2 header.
#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct EndHeaderTag {
    // u16 value
    typ: HeaderTagType,
    // u16 value
    flags: HeaderTagFlag,
    size: u32,
}

impl EndHeaderTag {
    pub fn new() -> Self {
        EndHeaderTag {
            typ: HeaderTagType::End,
            flags: HeaderTagFlag::Required,
            size: size_of::<Self>() as u32,
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
}

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
    requests: [TagType; N],
}

impl<const N: usize> InformationRequestHeaderTag<N> {
    pub fn new(flags: HeaderTagFlag, requests: [TagType; N]) -> Self {
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

    pub fn requests(&self) -> [TagType; N] {
        // cheap to clone, otherwise difficult with lifetime
        {self.requests}.clone()
    }

    pub fn req_iter(&self) -> InformationRequestHeaderTagIter {
        let base_size = size_of::<InformationRequestHeaderTag::<0>>() as u32;
        let count = (self.size - base_size) / size_of::<u32>() as u32;
        let base_ptr = self as *const InformationRequestHeaderTag::<N>;
        let base_ptr = base_ptr as *const u8;
        let base_ptr = unsafe { base_ptr.offset(base_size as isize) };
        let base_ptr = base_ptr as *const TagType;
        InformationRequestHeaderTagIter::new(
            count,
            base_ptr
        )
    }
}

impl <const N: usize> Debug for InformationRequestHeaderTag::<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("InformationRequestHeaderTag")
            .field("type", &{self.typ})
            .field("flags", &{self.flags})
            .field("size", &{self.size})
            // .field("requests", &)
            .field("req_iter", &{self.req_iter()})
            .finish()
    }
}

#[derive(Copy, Clone)]
pub struct InformationRequestHeaderTagIter<'a> {
    base_ptr: *const TagType,
    i: u32,
    count: u32,
    _marker: PhantomData<&'a ()>,
}

impl <'a> InformationRequestHeaderTagIter<'a> {
    fn new(count: u32, base_ptr: *const TagType) -> Self {
        Self {
            i: 0,
            count,
            base_ptr,
            _marker: PhantomData::default()
        }
    }
}

impl <'a> Iterator for InformationRequestHeaderTagIter<'a> {
    type Item = &'a TagType;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i < self.count {
            let ptr = unsafe { self.base_ptr.offset(self.i as isize) };
            self.i += 1;
            Some(unsafe {&*ptr})
        } else {
            None
        }
    }
}

impl <'a> Debug for InformationRequestHeaderTagIter<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_list();
        self.clone().for_each(|e| {
            debug.entry(e);
        });
        debug.finish()
    }
}



/// This information does not need to be provided if the kernel image is in ELF
/// format, but it must be provided if the image is in a.out format or in some
/// other format. Required for legacy boot (BIOS).
/// Determines load addresses.
#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct AddressHeaderTag {
    typ: HeaderTagType,
    flags: HeaderTagFlag,
    size: u32,
    /// Contains the address corresponding to the beginning of the Multiboot2 header — the physical memory location at which the magic value is supposed to be loaded. This field serves to synchronize the mapping between OS image offsets and physical memory addresses.
    header_addr: u32,
    /// Contains the physical address of the beginning of the text segment. The offset in the OS image file at which to start loading is defined by the offset at which the header was found, minus (header_addr - load_addr). load_addr must be less than or equal to header_addr.
    ///
    /// Special value -1 means that the file must be loaded from its beginning.
    load_addr: u32,
    /// Contains the physical address of the end of the data segment. (load_end_addr - load_addr) specifies how much data to load. This implies that the text and data segments must be consecutive in the OS image; this is true for existing a.out executable formats. If this field is zero, the boot loader assumes that the text and data segments occupy the whole OS image file.
    load_end_addr: u32,
    /// Contains the physical address of the end of the bss segment. The boot loader initializes this area to zero, and reserves the memory it occupies to avoid placing boot modules and other data relevant to the operating system in that area. If this field is zero, the boot loader assumes that no bss segment is present.
    bss_end_addr: u32,
}

impl AddressHeaderTag {
    pub fn new(
        flags: HeaderTagFlag,
        header_addr: u32,
        load_addr: u32,
        load_end_addr: u32,
        bss_end_addr: u32,
    ) -> Self {
        AddressHeaderTag {
            typ: HeaderTagType::Address,
            flags,
            size: size_of::<Self>() as u32,
            header_addr,
            load_addr,
            load_end_addr,
            bss_end_addr,
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
    pub fn header_addr(&self) -> u32 {
        self.header_addr
    }
    pub fn load_addr(&self) -> u32 {
        self.load_addr
    }
    pub fn load_end_addr(&self) -> u32 {
        self.load_end_addr
    }
    pub fn bss_end_addr(&self) -> u32 {
        self.bss_end_addr
    }
}

/// Specifies the physical address to which the boot loader should jump in
/// order to start running the operating system.
/// Not needed for ELF files.
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct EntryHeaderTag {
    typ: HeaderTagType,
    flags: HeaderTagFlag,
    size: u32,
    entry_addr: u32,
}

impl EntryHeaderTag {
    pub fn new(flags: HeaderTagFlag, entry_addr: u32) -> Self {
        EntryHeaderTag {
            typ: HeaderTagType::EntryAddress,
            flags,
            size: size_of::<Self>() as u32,
            entry_addr,
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
    pub fn entry_addr(&self) -> u32 {
        self.entry_addr
    }
}

impl Debug for EntryHeaderTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("EntryHeaderTag")
            .field("type", &{self.typ})
            .field("flags", &{self.flags})
            .field("size", &{self.size})
            .field("entry_addr", &(self.entry_addr as *const u32))
            .finish()
    }
}

/// Tells that a console must be available in MBI.
/// Only relevant for legacy BIOS.
#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct ConsoleHeaderTag {
    typ: HeaderTagType,
    flags: HeaderTagFlag,
    size: u32,
    console_flags: ConsoleHeaderTagFlags,
}

impl ConsoleHeaderTag {
    pub fn new(flags: HeaderTagFlag, console_flags: ConsoleHeaderTagFlags) -> Self {
        ConsoleHeaderTag {
            typ: HeaderTagType::ConsoleFlags,
            flags,
            size: size_of::<Self>() as u32,
            console_flags,
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
    pub fn console_flags(&self) -> ConsoleHeaderTagFlags {
        self.console_flags
    }
}

/// Specifies the preferred graphics mode. If this tag
/// is present the bootloader assumes that the payload
/// has framebuffer support. Note: This is only a
/// recommended mode. Only relevant on legacy BIOS.
#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct FramebufferHeaderTag {
    typ: HeaderTagType,
    flags: HeaderTagFlag,
    size: u32,
    width: u32,
    height: u32,
    depth: u32,
}

impl FramebufferHeaderTag {
    pub fn new(flags: HeaderTagFlag, width: u32, height: u32, depth: u32) -> Self {
        FramebufferHeaderTag {
            typ: HeaderTagType::Framebuffer,
            flags,
            size: size_of::<Self>() as u32,
            width,
            height,
            depth,
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
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
    pub fn depth(&self) -> u32 {
        self.depth
    }
}

/// If this tag is present modules must be page aligned.
#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct ModuleAlignHeaderTag {
    typ: HeaderTagType,
    flags: HeaderTagFlag,
    size: u32,
}

impl ModuleAlignHeaderTag {
    pub fn new(flags: HeaderTagFlag) -> Self {
        ModuleAlignHeaderTag {
            typ: HeaderTagType::ModuleAlign,
            flags,
            size: size_of::<Self>() as u32,
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
}

/// This tag indicates that payload supports starting without terminating UEFI boot services.
#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct EfiBootServiceHeaderTag {
    typ: HeaderTagType,
    flags: HeaderTagFlag,
    size: u32,
}

impl EfiBootServiceHeaderTag {
    pub fn new(flags: HeaderTagFlag) -> Self {
        EfiBootServiceHeaderTag {
            typ: HeaderTagType::EfiBS,
            flags,
            size: size_of::<Self>() as u32,
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
}

/// This tag is taken into account only on EFI i386 platforms when Multiboot2 image header
/// contains EFI boot services tag. Then entry point specified in ELF header and the entry address tag of Multiboot2 header are ignored.
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct EntryEfi32HeaderTag {
    typ: HeaderTagType,
    flags: HeaderTagFlag,
    size: u32,
    entry_addr: u32,
}

impl EntryEfi32HeaderTag {
    pub fn new(flags: HeaderTagFlag, entry_addr: u32) -> Self {
        EntryEfi32HeaderTag {
            typ: HeaderTagType::EntryAddressEFI32,
            flags,
            size: size_of::<Self>() as u32,
            entry_addr,
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
    pub fn entry_addr(&self) -> u32 {
        self.entry_addr
    }
}

impl Debug for EntryEfi32HeaderTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("EntryEfi32HeaderTag")
            .field("type", &{self.typ})
            .field("flags", &{self.flags})
            .field("size", &{self.size})
            .field("entry_addr", &(self.entry_addr as *const u32))
            .finish()
    }
}

/// This tag is taken into account only on EFI amd64 platforms when Multiboot2 image header
/// contains EFI boot services tag. Then entry point specified in ELF header and the entry address tag of Multiboot2 header are ignored.
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct EntryEfi64HeaderTag {
    typ: HeaderTagType,
    flags: HeaderTagFlag,
    size: u32,
    entry_addr: u32,
}

impl EntryEfi64HeaderTag {
    pub fn new(flags: HeaderTagFlag, entry_addr: u32) -> Self {
        EntryEfi64HeaderTag {
            typ: HeaderTagType::EntryAddressEFI64,
            flags,
            size: size_of::<Self>() as u32,
            entry_addr,
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
    pub fn entry_addr(&self) -> u32 {
        self.entry_addr
    }
}

impl Debug for EntryEfi64HeaderTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("EntryEfi64HeaderTag")
            .field("type", &{self.typ})
            .field("flags", &{self.flags})
            .field("size", &{self.size})
            .field("entry_addr", &(self.entry_addr as *const u32))
            .finish()
    }
}

/// This tag indicates that the image is relocatable.
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct RelocatableHeaderTag {
    typ: HeaderTagType,
    flags: HeaderTagFlag,
    size: u32,
    min_addr: u32,
    max_addr: u32,
    align: u32,
    preference: RelocatableHeaderTagPreference,
}

impl RelocatableHeaderTag {
    pub fn new(
        flags: HeaderTagFlag,
        min_addr: u32,
        max_addr: u32,
        align: u32,
        preference: RelocatableHeaderTagPreference,
    ) -> Self {
        RelocatableHeaderTag {
            typ: HeaderTagType::Relocatable,
            flags,
            size: size_of::<Self>() as u32,
            min_addr,
            max_addr,
            align,
            preference,
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
    pub fn min_addr(&self) -> u32 {
        self.min_addr
    }
    pub fn max_addr(&self) -> u32 {
        self.max_addr
    }
    pub fn align(&self) -> u32 {
        self.align
    }
    pub fn preference(&self) -> RelocatableHeaderTagPreference {
        self.preference
    }
}

impl Debug for RelocatableHeaderTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("RelocatableHeaderTag")
            .field("type", &{self.typ})
            .field("flags", &{self.flags})
            .field("size", &{self.size})
            // trick to print this as hexadecimal pointer
            .field("min_addr", &(self.min_addr as *const u32))
            .field("max_addr", &(self.max_addr as *const u32))
            .field("align", &{self.align})
            .field("preference", &{self.preference})
            .finish()
    }
}
