use crate::{
    AddressHeaderTag, ConsoleHeaderTag, EfiBootServiceHeaderTag, EntryAddressHeaderTag,
    EntryEfi32HeaderTag, EntryEfi64HeaderTag, FramebufferHeaderTag, HeaderTagHeader, HeaderTagISA,
    HeaderTagType, InformationRequestHeaderTag, ModuleAlignHeaderTag, RelocatableHeaderTag,
    TagIter,
};
use core::fmt::{Debug, Formatter};
use core::mem::size_of;
use core::ptr::NonNull;
use multiboot2_common::{ALIGNMENT, DynSizedStructure, Header, MemoryError, Tag};
use thiserror::Error;

/// Magic value for a [`Multiboot2Header`], as defined by the spec.
pub const MAGIC: u32 = 0xe85250d6;

/// Wrapper type around a pointer to the Multiboot2 header.
///
/// The Multiboot2 header is the [`Multiboot2BasicHeader`] followed
/// by all tags (see [`crate::tags::HeaderTagType`]).
/// Use this if you get a pointer to the header and just want
/// to parse it. If you want to construct the type by yourself,
/// please look at `HeaderBuilder` (requires the `builder` feature).
#[repr(transparent)]
pub struct Multiboot2Header<'a>(&'a DynSizedStructure<Multiboot2BasicHeader>);

impl<'a> Multiboot2Header<'a> {
    /// Public constructor for this type with various validations.
    ///
    /// If the header is invalid, it returns a [`LoadError`].
    /// This may be because:
    /// - `addr` is a null-pointer
    /// - `addr` isn't 8-byte aligned
    /// - the magic value of the header is not present
    /// - the checksum field is invalid
    ///
    /// # Safety
    /// This function may produce undefined behaviour, if the provided `addr` is not a valid
    /// Multiboot2 header pointer.
    pub unsafe fn load(ptr: *const Multiboot2BasicHeader) -> Result<Self, LoadError> {
        let ptr = NonNull::new(ptr.cast_mut()).ok_or(LoadError::Memory(MemoryError::Null))?;
        let inner = unsafe { DynSizedStructure::ref_from_ptr(ptr).map_err(LoadError::Memory)? };
        let this = Self(inner);

        let header = this.0.header();
        if header.header_magic != MAGIC {
            return Err(LoadError::MagicNotFound);
        }
        header
            .verify_checksum()
            .map_err(|x| LoadError::ChecksumMismatch(x.0, x.1))?;
        if !this.has_valid_end_tag() {
            return Err(LoadError::NoEndTag);
        }
        Ok(this)
    }

    /// Checks whether the header ends with the mandatory end tag.
    fn has_valid_end_tag(&self) -> bool {
        let payload = self.0.payload();
        if payload.len() < size_of::<HeaderTagHeader>() {
            return false;
        }

        let end_tag = &payload[payload.len() - size_of::<HeaderTagHeader>()..];
        let typ = u16::from_le_bytes(end_tag[0..2].try_into().unwrap());
        let flags = u16::from_le_bytes(end_tag[2..4].try_into().unwrap());
        let size = u32::from_le_bytes(end_tag[4..8].try_into().unwrap());

        typ == HeaderTagType::End as u16
            && flags == crate::HeaderTagFlag::Required as u16
            && size as usize == size_of::<HeaderTagHeader>()
    }

    /// Find the header in a given slice.
    ///
    /// If it succeeds, it returns a tuple consisting of the subslice containing
    /// just the header and the index of the header in the given slice.
    /// If it fails (either because the header is not properly 64-bit aligned
    /// or because it is truncated), it returns a [`LoadError`].
    /// If there is no header, it returns `None`.
    pub fn find_header(buffer: &[u8]) -> Result<Option<(&[u8], u32)>, LoadError> {
        if buffer.as_ptr().align_offset(ALIGNMENT) != 0 {
            return Err(LoadError::Memory(MemoryError::WrongAlignment));
        }

        let mut windows = buffer[0..8192].windows(4);
        let magic_index = match windows.position(|vals| {
            u32::from_le_bytes(vals.try_into().unwrap()) // yes, there's 4 bytes here
            == MAGIC
        }) {
            Some(idx) => {
                if idx % 8 == 0 {
                    idx
                } else {
                    return Err(LoadError::Memory(MemoryError::WrongAlignment));
                }
            }
            None => return Ok(None),
        };
        // skip over rest of magic
        windows.next();
        windows.next();
        windows.next();
        // arch
        windows.next();
        windows.next();
        windows.next();
        windows.next();
        let header_length: usize = u32::from_le_bytes(
            windows
                .next()
                .ok_or(LoadError::Memory(MemoryError::MissingPadding))?
                .try_into()
                .unwrap(), // 4 bytes are a u32
        )
        .try_into()
        .unwrap();
        Ok(Some((
            &buffer[magic_index..magic_index + header_length],
            magic_index as u32,
        )))
    }

    /// Returns a [`TagIter`].
    #[must_use]
    pub fn iter(&self) -> TagIter<'_> {
        TagIter::new(self.0.payload())
    }

    /// Wrapper around [`Multiboot2BasicHeader::verify_checksum`].
    pub const fn verify_checksum(
        &self,
    ) -> Result<
        (),
        (
            u32, /* actual checksum */
            u32, /* expected checksum */
        ),
    > {
        self.0.header().verify_checksum()
    }
    /// Wrapper around [`Multiboot2BasicHeader::header_magic`].
    #[must_use]
    pub const fn header_magic(&self) -> u32 {
        self.0.header().header_magic()
    }
    /// Wrapper around [`Multiboot2BasicHeader::arch`].
    #[must_use]
    pub const fn arch(&self) -> HeaderTagISA {
        self.0.header().arch()
    }
    /// Wrapper around [`Multiboot2BasicHeader::length`].
    #[must_use]
    pub const fn length(&self) -> u32 {
        self.0.header().length()
    }
    /// Wrapper around [`Multiboot2BasicHeader::checksum`].
    #[must_use]
    pub const fn checksum(&self) -> u32 {
        self.0.header().checksum()
    }
    /// Wrapper around [`Multiboot2BasicHeader::calc_checksum`].
    #[must_use]
    pub const fn calc_checksum(magic: u32, arch: HeaderTagISA, length: u32) -> u32 {
        Multiboot2BasicHeader::calc_checksum(magic, arch, length)
    }

    /// Search for the [`InformationRequestHeaderTag`] header tag.
    #[must_use]
    pub fn information_request_tag(&self) -> Option<&InformationRequestHeaderTag> {
        self.get_tag()
    }

    /// Search for the [`AddressHeaderTag`] header tag.
    #[must_use]
    pub fn address_tag(&self) -> Option<&AddressHeaderTag> {
        self.get_tag()
    }

    /// Search for the [`EntryAddressHeaderTag`] header tag.
    #[must_use]
    pub fn entry_address_tag(&self) -> Option<&EntryAddressHeaderTag> {
        self.get_tag()
    }

    /// Search for the [`EntryEfi32HeaderTag`] header tag.
    #[must_use]
    pub fn entry_address_efi32_tag(&self) -> Option<&EntryEfi32HeaderTag> {
        self.get_tag()
    }

    /// Search for the [`EntryEfi64HeaderTag`] header tag.
    #[must_use]
    pub fn entry_address_efi64_tag(&self) -> Option<&EntryEfi64HeaderTag> {
        self.get_tag()
    }

    /// Search for the [`ConsoleHeaderTag`] header tag.
    #[must_use]
    pub fn console_flags_tag(&self) -> Option<&ConsoleHeaderTag> {
        self.get_tag()
    }

    /// Search for the [`FramebufferHeaderTag`] header tag.
    #[must_use]
    pub fn framebuffer_tag(&self) -> Option<&FramebufferHeaderTag> {
        self.get_tag()
    }

    /// Search for the [`ModuleAlignHeaderTag`] header tag.
    #[must_use]
    pub fn module_align_tag(&self) -> Option<&ModuleAlignHeaderTag> {
        self.get_tag()
    }

    /// Search for the [`EfiBootServiceHeaderTag`] header tag.
    #[must_use]
    pub fn efi_boot_services_tag(&self) -> Option<&EfiBootServiceHeaderTag> {
        self.get_tag()
    }

    /// Search for the [`RelocatableHeaderTag`] header tag.
    #[must_use]
    pub fn relocatable_tag(&self) -> Option<&RelocatableHeaderTag> {
        self.get_tag()
    }

    /// Searches for the specified tag by iterating the structure and returns
    /// the first occurrence, if present.
    #[must_use]
    fn get_tag<T: Tag<IDType = HeaderTagType, Header = HeaderTagHeader> + ?Sized + 'a>(
        &'a self,
    ) -> Option<&'a T> {
        self.iter()
            .find(|tag| tag.header().typ() == T::ID)
            .map(|tag| tag.cast::<T>())
    }
}

impl Debug for Multiboot2Header<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Multiboot2Header")
            .field("magic", &self.header_magic())
            .field("arch", &self.arch())
            .field("length", &self.length())
            .field("checksum", &self.checksum())
            // TODO better debug impl
            .field("tags", &"<tags iter>")
            .finish()
    }
}

/// Errors that occur when a chunk of memory can't be parsed as
/// [`Multiboot2Header`].
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Error)]
pub enum LoadError {
    /// The provided checksum does not match the expected value.
    #[error("checksum 0x{0:X} does not match expected value 0x{1:x}")]
    ChecksumMismatch(u32 /* is */, u32 /* expected */),
    /// The header does not contain the correct magic number.
    #[error("header does not contain expected magic value")]
    MagicNotFound,
    /// Missing mandatory end tag.
    #[error("missing mandatory end tag")]
    NoEndTag,
    /// The provided memory can't be parsed as [`Multiboot2Header`].
    /// See [`MemoryError`].
    #[error("memory can't be parsed as multiboot2 header")]
    Memory(#[source] MemoryError),
}

/// The "basic" Multiboot2 header. This means only the properties, that are known during
/// compile time. All other information are derived during runtime from the size property.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C, align(8))]
pub struct Multiboot2BasicHeader {
    /// Must be the value of [`MAGIC`].
    header_magic: u32,
    arch: HeaderTagISA,
    length: u32,
    checksum: u32,
    // Followed by dynamic amount of dynamically sized header tags.
    // At minimum, the end tag.
}

impl Multiboot2BasicHeader {
    #[cfg(feature = "builder")]
    /// Constructor for the basic header.
    pub(crate) const fn new(arch: HeaderTagISA, length: u32) -> Self {
        let magic = MAGIC;
        let checksum = Self::calc_checksum(magic, arch, length);
        Self {
            header_magic: magic,
            arch,
            length,
            checksum,
        }
    }

    /// Verifies if a Multiboot2 header is valid.
    pub const fn verify_checksum(
        &self,
    ) -> Result<
        (),
        (
            u32, /* actual checksum */
            u32, /* expected checksum */
        ),
    > {
        let check = Self::calc_checksum(self.header_magic, self.arch, self.length);
        if check == self.checksum {
            Ok(())
        } else {
            Err((self.checksum, check))
        }
    }

    /// Calculates the checksum as described in the spec.
    #[must_use]
    pub const fn calc_checksum(magic: u32, arch: HeaderTagISA, length: u32) -> u32 {
        (0x100000000 - magic as u64 - arch as u64 - length as u64) as u32
    }

    /// Returns the header magic.
    #[must_use]
    pub const fn header_magic(&self) -> u32 {
        self.header_magic
    }

    /// Returns the [`HeaderTagISA`].
    #[must_use]
    pub const fn arch(&self) -> HeaderTagISA {
        self.arch
    }

    /// Returns the length.
    #[must_use]
    pub const fn length(&self) -> u32 {
        self.length
    }

    /// Returns the checksum.
    #[must_use]
    pub const fn checksum(&self) -> u32 {
        self.checksum
    }
}

impl Header for Multiboot2BasicHeader {
    fn payload_len(&self) -> usize {
        self.length as usize - size_of::<Self>()
    }

    fn set_size(&mut self, total_size: usize) {
        self.length = total_size as u32;
        self.checksum = Self::calc_checksum(self.header_magic, self.arch, total_size as u32);
    }
}

impl Debug for Multiboot2BasicHeader {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Multiboot2Header")
            .field("header_magic", &{ self.header_magic })
            .field("arch", &{ self.arch })
            .field("length", &{ self.length })
            .field("checksum", &{ self.checksum })
            //.field("tags", &self.iter())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::{HeaderTagISA, LoadError, MAGIC, Multiboot2BasicHeader, Multiboot2Header};
    use multiboot2_common::test_utils::AlignedBytes;

    /// Writes a minimal valid Multiboot2 header into the buffer, consisting
    /// only of the basic header and an end tag.
    fn write_minimal_valid_header_tag(buffer: &mut [u8]) {
        // Aligned magic
        buffer[0..4].copy_from_slice(&MAGIC.to_le_bytes());
        // Architecture
        buffer[4..8].copy_from_slice(&(HeaderTagISA::I386 as u32).to_le_bytes());
        // Total size
        buffer[8..12].copy_from_slice(&24_u32.to_le_bytes());
        // Checksum
        buffer[12..16].copy_from_slice(&0x17adaf12_u32.to_le_bytes());
        // End tag: ID
        buffer[16..18].copy_from_slice(&0_u16.to_le_bytes());
        // End tag: Flags
        buffer[18..20].copy_from_slice(&0_u16.to_le_bytes());
        // End tag: Size
        buffer[20..24].copy_from_slice(&8_u32.to_le_bytes());
    }

    #[test]
    fn test_assert_size() {
        assert_eq!(core::mem::size_of::<Multiboot2BasicHeader>(), 4 + 4 + 4 + 4);
    }

    #[test]
    fn load_accepts_minimal_header_with_end_tag() {
        let mut bytes = AlignedBytes::new([0; 24]);
        write_minimal_valid_header_tag(&mut bytes.0);

        let header = unsafe { Multiboot2Header::load(bytes.as_ptr().cast()) };

        assert!(header.is_ok());
    }

    #[test]
    fn load_rejects_missing_end_tag() {
        let mut bytes = AlignedBytes::new([0; 16]);
        let checksum = Multiboot2BasicHeader::calc_checksum(MAGIC, HeaderTagISA::I386, 16);
        bytes.0[0..4].copy_from_slice(&MAGIC.to_le_bytes());
        bytes.0[8..12].copy_from_slice(&16_u32.to_le_bytes());
        bytes.0[12..16].copy_from_slice(&checksum.to_le_bytes());

        let header = unsafe { Multiboot2Header::load(bytes.as_ptr().cast()) };

        assert!(matches!(header, Err(LoadError::NoEndTag)));
    }
}
