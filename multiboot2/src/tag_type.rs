//! Module for the basic Multiboot2 tag and corresponding tag types.
//!
//! The relevant exports of this module are:
//! - [`EndTag`]
//! - [`TagTypeId`]
//! - [`TagType`]
//! - [`Tag`]
#[cfg(feature = "builder")]
use crate::builder::traits::StructAsBytes;

use crate::TagTrait;
use core::fmt::{Debug, Formatter};
use core::hash::Hash;
use core::marker::PhantomData;
use core::str::Utf8Error;

/// Serialized form of [`TagType`] that matches the binary representation
/// (`u32`). The abstraction corresponds to the `typ`/`type` field of a
/// Multiboot2 [`Tag`]. This type can easily be created from or converted to
/// [`TagType`].
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Eq, Ord, Hash)]
pub struct TagTypeId(u32);

impl TagTypeId {
    /// Constructor.
    pub fn new(val: u32) -> Self {
        Self(val)
    }
}

/// Higher level abstraction for [`TagTypeId`] that assigns each possible value
/// to a specific semantic according to the specification. Additionally, it
/// allows to use the [`TagType::Custom`] variant. It is **not binary compatible**
/// with [`TagTypeId`].
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TagType {
    /// Tag `0`: Marks the end of the tags.
    End,
    /// Tag `1`: Additional command line string.
    /// For example `''` or `'--my-custom-option foo --provided by_grub`, if
    /// your GRUB config contains `multiboot2 /boot/multiboot2-binary.elf --my-custom-option foo --provided by_grub`
    Cmdline,
    /// Tag `2`: Name of the bootloader, e.g. 'GRUB 2.04-1ubuntu44.2'
    BootLoaderName,
    /// Tag `3`: Additional Multiboot modules, which are BLOBs provided in
    /// memory. For example an initial ram disk with essential drivers.
    Module,
    /// Tag `4`: ‘mem_lower’ and ‘mem_upper’ indicate the amount of lower and
    /// upper memory, respectively, in kilobytes. Lower memory starts at
    /// address 0, and upper memory starts at address 1 megabyte. The maximum
    /// possible value for lower memory is 640 kilobytes. The value returned
    /// for upper memory is maximally the address of the first upper memory
    /// hole minus 1 megabyte. It is not guaranteed to be this value.
    ///
    /// This tag may not be provided by some boot loaders on EFI platforms if
    /// EFI boot services are enabled and available for the loaded image (EFI
    /// boot services not terminated tag exists in Multiboot2 information
    /// structure).
    BasicMeminfo,
    /// Tag `5`: This tag indicates which BIOS disk device the boot loader
    /// loaded the OS image from. If the OS image was not loaded from a BIOS
    /// disk, then this tag must not be present. The operating system may use
    /// this field as a hint for determining its own root device, but is not
    /// required to.
    Bootdev,
    /// Tag `6`: Memory map. The map provided is guaranteed to list all
    /// standard RAM that should be available for normal use. This type however
    /// includes the regions occupied by kernel, mbi, segments and modules.
    /// Kernel must take care not to overwrite these regions.
    ///
    /// This tag may not be provided by some boot loaders on EFI platforms if
    /// EFI boot services are enabled and available for the loaded image (EFI
    /// boot services not terminated tag exists in Multiboot2 information
    /// structure).
    Mmap,
    /// Tag `7`: Contains the VBE control information returned by the VBE
    /// Function `0x00` and VBE mode information returned by the VBE Function
    /// `0x01`, respectively. Note that VBE 3.0 defines another protected mode
    /// interface which is incompatible with the old one. If you want to use
    /// the new protected mode interface, you will have to find the table
    /// yourself.
    Vbe,
    /// Tag `8`: Framebuffer.
    Framebuffer,
    /// Tag `9`: This tag contains section header table from an ELF kernel, the
    /// size of each entry, number of entries, and the string table used as the
    /// index of names. They correspond to the `shdr_*` entries (`shdr_num`,
    /// etc.) in the Executable and Linkable Format (ELF) specification in the
    /// program header.
    ElfSections,
    /// Tag `10`: APM table. See Advanced Power Management (APM) BIOS Interface
    /// Specification, for more information.
    Apm,
    /// Tag `11`: This tag contains pointer to i386 EFI system table.
    Efi32,
    /// Tag `21`: This tag contains pointer to amd64 EFI system table.
    Efi64,
    /// Tag `13`: This tag contains a copy of SMBIOS tables as well as their
    /// version.
    Smbios,
    /// Tag `14`: Also called "AcpiOld" in other multiboot2 implementations.
    AcpiV1,
    /// Tag `15`: Refers to version 2 and later of Acpi.
    /// Also called "AcpiNew" in other multiboot2 implementations.
    AcpiV2,
    /// Tag `16`: This tag contains network information in the format specified
    /// as DHCP. It may be either a real DHCP reply or just the configuration
    /// info in the same format. This tag appears once
    /// per card.
    Network,
    /// Tag `17`: This tag contains EFI memory map as per EFI specification.
    /// This tag may not be provided by some boot loaders on EFI platforms if
    /// EFI boot services are enabled and available for the loaded image (EFI
    /// boot services not terminated tag exists in Multiboot2 information
    /// structure).
    EfiMmap,
    /// Tag `18`: This tag indicates ExitBootServices wasn't called.
    EfiBs,
    /// Tag `19`: This tag contains pointer to EFI i386 image handle. Usually
    /// it is boot loader image handle.
    Efi32Ih,
    /// Tag `20`: This tag contains pointer to EFI amd64 image handle. Usually
    /// it is boot loader image handle.
    Efi64Ih,
    /// Tag `21`: This tag contains image load base physical address. The spec
    /// tells *"It is provided only if image has relocatable header tag."* but
    /// experience showed that this is not true for at least GRUB 2.
    LoadBaseAddr,
    /// Custom tag types `> 21`. The Multiboot2 spec doesn't explicitly allow
    /// or disallow them. Bootloader and OS developers are free to use custom
    /// tags.
    Custom(u32),
}

impl TagType {
    /// Convenient wrapper to get the underlying `u32` representation of the tag.
    pub fn val(&self) -> u32 {
        u32::from(*self)
    }
}

/// Relevant `From`-implementations for conversions between `u32`, [´TagTypeId´]
/// and [´TagType´].
mod primitive_conversion_impls {
    use super::*;

    impl From<u32> for TagTypeId {
        fn from(value: u32) -> Self {
            // SAFETY: the type has repr(transparent)
            unsafe { core::mem::transmute(value) }
        }
    }

    impl From<TagTypeId> for u32 {
        fn from(value: TagTypeId) -> Self {
            value.0 as _
        }
    }

    impl From<u32> for TagType {
        fn from(value: u32) -> Self {
            match value {
                0 => TagType::End,
                1 => TagType::Cmdline,
                2 => TagType::BootLoaderName,
                3 => TagType::Module,
                4 => TagType::BasicMeminfo,
                5 => TagType::Bootdev,
                6 => TagType::Mmap,
                7 => TagType::Vbe,
                8 => TagType::Framebuffer,
                9 => TagType::ElfSections,
                10 => TagType::Apm,
                11 => TagType::Efi32,
                12 => TagType::Efi64,
                13 => TagType::Smbios,
                14 => TagType::AcpiV1,
                15 => TagType::AcpiV2,
                16 => TagType::Network,
                17 => TagType::EfiMmap,
                18 => TagType::EfiBs,
                19 => TagType::Efi32Ih,
                20 => TagType::Efi64Ih,
                21 => TagType::LoadBaseAddr,
                c => TagType::Custom(c),
            }
        }
    }

    impl From<TagType> for u32 {
        fn from(value: TagType) -> Self {
            match value {
                TagType::End => 0,
                TagType::Cmdline => 1,
                TagType::BootLoaderName => 2,
                TagType::Module => 3,
                TagType::BasicMeminfo => 4,
                TagType::Bootdev => 5,
                TagType::Mmap => 6,
                TagType::Vbe => 7,
                TagType::Framebuffer => 8,
                TagType::ElfSections => 9,
                TagType::Apm => 10,
                TagType::Efi32 => 11,
                TagType::Efi64 => 12,
                TagType::Smbios => 13,
                TagType::AcpiV1 => 14,
                TagType::AcpiV2 => 15,
                TagType::Network => 16,
                TagType::EfiMmap => 17,
                TagType::EfiBs => 18,
                TagType::Efi32Ih => 19,
                TagType::Efi64Ih => 20,
                TagType::LoadBaseAddr => 21,
                TagType::Custom(c) => c,
            }
        }
    }
}

/// `From`-implementations for conversions between [´TagTypeId´] and [´TagType´].
mod intermediate_conversion_impls {
    use super::*;

    impl From<TagTypeId> for TagType {
        fn from(value: TagTypeId) -> Self {
            let value = u32::from(value);
            TagType::from(value)
        }
    }

    impl From<TagType> for TagTypeId {
        fn from(value: TagType) -> Self {
            let value = u32::from(value);
            TagTypeId::from(value)
        }
    }
}

/// Implements `partial_eq` between [´TagTypeId´] and [´TagType´]. Two values
/// are equal if their `u32` representation is equal. Additionally, `u32` can
/// be compared with [´TagTypeId´].
mod partial_eq_impls {
    use super::*;

    impl PartialEq<TagTypeId> for TagType {
        fn eq(&self, other: &TagTypeId) -> bool {
            let this = u32::from(*self);
            let that = u32::from(*other);
            this == that
        }
    }

    // each compare/equal direction must be implemented manually
    impl PartialEq<TagType> for TagTypeId {
        fn eq(&self, other: &TagType) -> bool {
            other.eq(self)
        }
    }

    impl PartialEq<u32> for TagTypeId {
        fn eq(&self, other: &u32) -> bool {
            let this = u32::from(*self);
            this == *other
        }
    }

    impl PartialEq<TagTypeId> for u32 {
        fn eq(&self, other: &TagTypeId) -> bool {
            other.eq(self)
        }
    }

    impl PartialEq<u32> for TagType {
        fn eq(&self, other: &u32) -> bool {
            let this = u32::from(*self);
            this == *other
        }
    }

    impl PartialEq<TagType> for u32 {
        fn eq(&self, other: &TagType) -> bool {
            other.eq(self)
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
    // additional, tag specific fields
}

impl Tag {
    /// Casts the base tag to the specific tag type.
    pub fn cast_tag<'a, T: TagTrait + ?Sized>(&self) -> &'a T {
        // Safety: At this point, we trust that "self.size" and the size hint
        // for DST tags are sane.
        unsafe { TagTrait::from_base_tag(self) }
    }

    /// Some multiboot2 tags are a DST as they end with a dynamically sized byte
    /// slice. This function parses this slice as [`str`] so that either a valid
    /// UTF-8 Rust string slice without a terminating null byte or an error is
    /// returned.
    pub fn get_dst_str_slice(bytes: &[u8]) -> Result<&str, Utf8Error> {
        if bytes.is_empty() {
            // Very unlikely. A sane bootloader would omit the tag in this case.
            // But better be safe.
            return Ok("");
        }

        // Return without a trailing null byte. By spec, the null byte should
        // always terminate the string. However, for safety, we do make an extra
        // check.
        let str_slice = if bytes.ends_with(&[b'\0']) {
            let str_len = bytes.len() - 1;
            &bytes[0..str_len]
        } else {
            // Unlikely that a bootloader doesn't follow the spec and does not
            // add a terminating null byte.
            bytes
        };
        core::str::from_utf8(str_slice)
    }
}

impl Debug for Tag {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
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

/// The end tag ends the information struct.
#[repr(C)]
#[derive(Debug)]
pub struct EndTag {
    pub typ: TagTypeId,
    pub size: u32,
}

impl Default for EndTag {
    fn default() -> Self {
        Self {
            typ: TagType::End.into(),
            size: 8,
        }
    }
}

#[cfg(feature = "builder")]
impl StructAsBytes for EndTag {
    fn byte_size(&self) -> usize {
        core::mem::size_of::<Self>()
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
        match tag {
            &Tag {
                // END-Tag
                typ: TagTypeId(0),
                size: 8,
            } => None, // end tag
            tag => {
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
    use std::mem::{align_of, size_of};

    #[test]
    fn test_hashset() {
        let mut set = std::collections::HashSet::new();
        set.insert(TagType::Cmdline);
        set.insert(TagType::ElfSections);
        set.insert(TagType::BootLoaderName);
        set.insert(TagType::LoadBaseAddr);
        set.insert(TagType::LoadBaseAddr);
        assert_eq!(set.len(), 4);
        println!("{:#?}", set);
    }

    #[test]
    fn test_btreeset() {
        let mut set = std::collections::BTreeSet::new();
        set.insert(TagType::Cmdline);
        set.insert(TagType::ElfSections);
        set.insert(TagType::BootLoaderName);
        set.insert(TagType::LoadBaseAddr);
        set.insert(TagType::LoadBaseAddr);
        assert_eq!(set.len(), 4);
        for (current, next) in set.iter().zip(set.iter().skip(1)) {
            assert!(current < next);
        }
        println!("{:#?}", set);
    }

    /// Tests for equality when one type is u32 and the other the enum representation.
    #[test]
    fn test_partial_eq_u32() {
        assert_eq!(21, TagType::LoadBaseAddr);
        assert_eq!(TagType::LoadBaseAddr, 21);
        assert_eq!(21, TagTypeId(21));
        assert_eq!(TagTypeId(21), 21);
        assert_eq!(42, TagType::Custom(42));
        assert_eq!(TagType::Custom(42), 42);
    }

    /// Tests the construction of [`TagTypeId`] from primitive `u32` values.
    #[test]
    #[allow(non_snake_case)]
    fn test_TagTypeId() {
        assert_eq!(size_of::<TagTypeId>(), size_of::<u32>());
        assert_eq!(align_of::<TagTypeId>(), align_of::<u32>());

        for i in 0..50_u32 {
            let val: TagTypeId = i.into();
            let val2: TagType = val.into();
            assert_eq!(val, val2);
        }

        let tag_custom: u32 = 0x1337;
        let tag_custom: TagTypeId = tag_custom.into();
        let tag_custom: TagType = tag_custom.into();
        matches!(tag_custom, TagType::Custom(0x1337));
    }

    /// Tests the construction of [`TagTypeId`] from primitive `u32` values for
    /// specified and custom tags.
    #[test]
    #[allow(non_snake_case)]
    fn test_from_and_to_tag_type_id() {
        for i in 0..1_000 {
            let tag_type_id = TagTypeId::new(i);
            let tag_type_from_id = TagType::from(tag_type_id);
            let tag_type_from_u16 = TagType::from(i);
            assert_eq!(tag_type_from_id, tag_type_from_u16)
        }
    }

    #[test]
    fn test_get_dst_str_slice() {
        // unlikely case
        assert_eq!(Ok(""), Tag::get_dst_str_slice(&[]));
        // also unlikely case
        assert_eq!(Ok(""), Tag::get_dst_str_slice(&[b'\0']));
        // unlikely case: missing null byte. but the lib can cope with that
        assert_eq!(Ok("foobar"), Tag::get_dst_str_slice("foobar".as_bytes()));
        // test that the null bytes is not included in the string slice
        assert_eq!(Ok("foobar"), Tag::get_dst_str_slice("foobar\0".as_bytes()));
        // test invalid utf8
        assert!(matches!(
            Tag::get_dst_str_slice(&[0xff, 0xff]),
            Err(Utf8Error { .. })
        ));
    }

    #[test]
    /// Compile time test for [`EndTag`].
    fn test_end_tag_size() {
        unsafe {
            core::mem::transmute::<[u8; 8], EndTag>([0u8; 8]);
        }
    }
}
