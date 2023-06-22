//! Exports item [`HeaderBuilder`].

use crate::builder::information_request::InformationRequestHeaderTagBuilder;
use crate::builder::traits::StructAsBytes;
use crate::HeaderTagISA;
use crate::{
    AddressHeaderTag, ConsoleHeaderTag, EfiBootServiceHeaderTag, EndHeaderTag,
    EntryAddressHeaderTag, EntryEfi32HeaderTag, EntryEfi64HeaderTag, FramebufferHeaderTag,
    ModuleAlignHeaderTag, Multiboot2BasicHeader, RelocatableHeaderTag,
};
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::mem::size_of;
use core::ops::Deref;

/// Holds the raw bytes of a boot information built with [`HeaderBuilder`]
/// on the heap. The bytes returned by [`HeaderBytes::as_bytes`] are
/// guaranteed to be properly aligned.
#[derive(Clone, Debug)]
pub struct HeaderBytes {
    // Offset into the bytes where the header starts. This is necessary to
    // guarantee alignment at the moment.
    offset: usize,
    bytes: Box<[u8]>,
}

impl HeaderBytes {
    /// Returns the bytes. They are guaranteed to be correctly aligned.
    pub fn as_bytes(&self) -> &[u8] {
        let slice = &self.bytes[self.offset..];
        // At this point, the alignment is guaranteed. If not, something is
        // broken fundamentally.
        assert_eq!(slice.as_ptr().align_offset(8), 0);
        slice
    }
}

impl Deref for HeaderBytes {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_bytes()
    }
}

/// Builder to construct a valid Multiboot2 header dynamically at runtime.
/// The tags will appear in the order of their corresponding enumeration,
/// except for the END tag.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeaderBuilder {
    arch: HeaderTagISA,
    // first
    information_request_tag: Option<InformationRequestHeaderTagBuilder>,
    // second
    address_tag: Option<AddressHeaderTag>,
    // third
    entry_tag: Option<EntryAddressHeaderTag>,
    // fourth
    console_tag: Option<ConsoleHeaderTag>,
    // fifth
    framebuffer_tag: Option<FramebufferHeaderTag>,
    // sixth
    module_align_tag: Option<ModuleAlignHeaderTag>,
    // seventh
    efi_bs_tag: Option<EfiBootServiceHeaderTag>,
    // eighth
    efi_32_tag: Option<EntryEfi32HeaderTag>,
    // ninth
    efi_64_tag: Option<EntryEfi64HeaderTag>,
    // tenth (last)
    relocatable_tag: Option<RelocatableHeaderTag>,
}

impl HeaderBuilder {
    pub const fn new(arch: HeaderTagISA) -> Self {
        Self {
            arch,
            information_request_tag: None,
            address_tag: None,
            entry_tag: None,
            console_tag: None,
            framebuffer_tag: None,
            module_align_tag: None,
            efi_bs_tag: None,
            efi_32_tag: None,
            efi_64_tag: None,
            relocatable_tag: None,
        }
    }

    /// Returns the size, if the value is a multiple of 8 or returns
    /// the next number that is a multiple of 8. With this, one can
    /// easily calculate the size of a Multiboot2 header, where
    /// all the tags are 8-byte aligned.
    const fn size_or_up_aligned(size: usize) -> usize {
        (size + 7) & !7
    }

    /// Returns the expected length of the Multiboot2 header, when the
    /// [`Self::build`]-method gets called.
    pub fn expected_len(&self) -> usize {
        let base_len = size_of::<Multiboot2BasicHeader>();
        // size_or_up_aligned not required, because basic header length is 16 and the
        // begin is 8 byte aligned => first tag automatically 8 byte aligned
        let mut len = Self::size_or_up_aligned(base_len);
        if let Some(tag_builder) = self.information_request_tag.as_ref() {
            // we use size_or_up_aligned, because each tag will start at an 8 byte aligned address.
            // Attention: expected len from builder, not the size of the builder itself!
            len += Self::size_or_up_aligned(tag_builder.expected_len())
        }
        if self.address_tag.is_some() {
            // we use size_or_up_aligned, because each tag will start at an 8 byte aligned address
            len += Self::size_or_up_aligned(size_of::<AddressHeaderTag>())
        }
        if self.entry_tag.is_some() {
            len += Self::size_or_up_aligned(size_of::<EntryAddressHeaderTag>())
        }
        if self.console_tag.is_some() {
            len += Self::size_or_up_aligned(size_of::<ConsoleHeaderTag>())
        }
        if self.framebuffer_tag.is_some() {
            len += Self::size_or_up_aligned(size_of::<FramebufferHeaderTag>())
        }
        if self.module_align_tag.is_some() {
            len += Self::size_or_up_aligned(size_of::<ModuleAlignHeaderTag>())
        }
        if self.efi_bs_tag.is_some() {
            len += Self::size_or_up_aligned(size_of::<EfiBootServiceHeaderTag>())
        }
        if self.efi_32_tag.is_some() {
            len += Self::size_or_up_aligned(size_of::<EntryEfi32HeaderTag>())
        }
        if self.efi_64_tag.is_some() {
            len += Self::size_or_up_aligned(size_of::<EntryEfi64HeaderTag>())
        }
        if self.relocatable_tag.is_some() {
            len += Self::size_or_up_aligned(size_of::<RelocatableHeaderTag>())
        }
        // only here size_or_up_aligned is not important, because it is the last tag
        len += size_of::<EndHeaderTag>();
        len
    }

    /// Adds the bytes of a tag to the final Multiboot2 header byte vector.
    fn build_add_bytes(dest: &mut Vec<u8>, source: &[u8], is_end_tag: bool) {
        let vec_next_write_ptr = unsafe { dest.as_ptr().add(dest.len()) };
        // At this point, the alignment is guaranteed. If not, something is
        // broken fundamentally.
        assert_eq!(vec_next_write_ptr.align_offset(8), 0);

        dest.extend(source);
        if !is_end_tag {
            let size = source.len();
            let size_to_8_align = Self::size_or_up_aligned(size);
            let size_to_8_align_diff = size_to_8_align - size;
            // fill zeroes so that next data block is 8-byte aligned
            dest.extend([0].repeat(size_to_8_align_diff));
        }
    }

    /// Constructs the bytes for a valid Multiboot2 header with the given properties.
    pub fn build(mut self) -> HeaderBytes {
        const ALIGN: usize = 8;

        // PHASE 1/3: Prepare Vector

        // We allocate more than necessary so that we can ensure an correct
        // alignment within this data.
        let alloc_len = self.expected_len() + 7;
        let mut bytes = Vec::<u8>::with_capacity(alloc_len);
        // Pointer to check that no relocation happened.
        let alloc_ptr = bytes.as_ptr();

        // As long as there is no nice way in stable Rust to guarantee the
        // alignment of a vector, I add zero bytes at the beginning and the
        // header might not start at the start of the allocation.
        //
        // Unfortunately, it is not possible to reliably test this in a unit
        // test as long as the allocator_api feature is not stable.
        // Due to my manual testing, however, it works.
        let offset = bytes.as_ptr().align_offset(ALIGN);
        bytes.extend([0].repeat(offset));

        // -----------------------------------------------
        // PHASE 2/3: Add Tags
        self.build_add_tags(&mut bytes);

        // -----------------------------------------------
        // PHASE 3/3: Finalize Vector

        // Ensure that the vector has the same length as it's capacity. This is
        // important so that miri doesn't complain that the boxed memory is
        // smaller than the original allocation.
        bytes.extend([0].repeat(bytes.capacity() - bytes.len()));

        assert_eq!(
            alloc_ptr,
            bytes.as_ptr(),
            "Vector was reallocated. Alignment of header probably broken!"
        );
        assert_eq!(
            bytes[0..offset].iter().sum::<u8>(),
            0,
            "The offset to alignment area should be zero."
        );

        // Construct a box from a vec without `into_boxed_slice`. The latter
        // calls `shrink` on the allocator, which might reallocate this memory.
        // We don't want that!
        let bytes = unsafe { Box::from_raw(bytes.leak()) };

        assert_eq!(bytes.len(), alloc_len);

        HeaderBytes { offset, bytes }
    }

    /// Helper method that adds all the tags to the given vector.
    fn build_add_tags(&mut self, bytes: &mut Vec<u8>) {
        Self::build_add_bytes(
            bytes,
            // important that we write the correct expected length into the header!
            &Multiboot2BasicHeader::new(self.arch, self.expected_len() as u32).struct_as_bytes(),
            false,
        );

        if let Some(irs) = self.information_request_tag.take() {
            Self::build_add_bytes(bytes, &irs.build(), false)
        }
        if let Some(tag) = self.address_tag.as_ref() {
            Self::build_add_bytes(bytes, &tag.struct_as_bytes(), false)
        }
        if let Some(tag) = self.entry_tag.as_ref() {
            Self::build_add_bytes(bytes, &tag.struct_as_bytes(), false)
        }
        if let Some(tag) = self.console_tag.as_ref() {
            Self::build_add_bytes(bytes, &tag.struct_as_bytes(), false)
        }
        if let Some(tag) = self.framebuffer_tag.as_ref() {
            Self::build_add_bytes(bytes, &tag.struct_as_bytes(), false)
        }
        if let Some(tag) = self.module_align_tag.as_ref() {
            Self::build_add_bytes(bytes, &tag.struct_as_bytes(), false)
        }
        if let Some(tag) = self.efi_bs_tag.as_ref() {
            Self::build_add_bytes(bytes, &tag.struct_as_bytes(), false)
        }
        if let Some(tag) = self.efi_32_tag.as_ref() {
            Self::build_add_bytes(bytes, &tag.struct_as_bytes(), false)
        }
        if let Some(tag) = self.efi_64_tag.as_ref() {
            Self::build_add_bytes(bytes, &tag.struct_as_bytes(), false)
        }
        if let Some(tag) = self.relocatable_tag.as_ref() {
            Self::build_add_bytes(bytes, &tag.struct_as_bytes(), false)
        }
        Self::build_add_bytes(bytes, &EndHeaderTag::new().struct_as_bytes(), true);
    }

    // clippy thinks this can be a const fn but the compiler denies it
    #[allow(clippy::missing_const_for_fn)]
    pub fn information_request_tag(
        mut self,
        information_request_tag: InformationRequestHeaderTagBuilder,
    ) -> Self {
        self.information_request_tag = Some(information_request_tag);
        self
    }
    pub const fn address_tag(mut self, address_tag: AddressHeaderTag) -> Self {
        self.address_tag = Some(address_tag);
        self
    }
    pub const fn entry_tag(mut self, entry_tag: EntryAddressHeaderTag) -> Self {
        self.entry_tag = Some(entry_tag);
        self
    }
    pub const fn console_tag(mut self, console_tag: ConsoleHeaderTag) -> Self {
        self.console_tag = Some(console_tag);
        self
    }
    pub const fn framebuffer_tag(mut self, framebuffer_tag: FramebufferHeaderTag) -> Self {
        self.framebuffer_tag = Some(framebuffer_tag);
        self
    }
    pub const fn module_align_tag(mut self, module_align_tag: ModuleAlignHeaderTag) -> Self {
        self.module_align_tag = Some(module_align_tag);
        self
    }
    pub const fn efi_bs_tag(mut self, efi_bs_tag: EfiBootServiceHeaderTag) -> Self {
        self.efi_bs_tag = Some(efi_bs_tag);
        self
    }
    pub const fn efi_32_tag(mut self, efi_32_tag: EntryEfi32HeaderTag) -> Self {
        self.efi_32_tag = Some(efi_32_tag);
        self
    }
    pub const fn efi_64_tag(mut self, efi_64_tag: EntryEfi64HeaderTag) -> Self {
        self.efi_64_tag = Some(efi_64_tag);
        self
    }
    pub const fn relocatable_tag(mut self, relocatable_tag: RelocatableHeaderTag) -> Self {
        self.relocatable_tag = Some(relocatable_tag);
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::builder::header::HeaderBuilder;
    use crate::builder::information_request::InformationRequestHeaderTagBuilder;
    use crate::{
        HeaderTagFlag, HeaderTagISA, MbiTagType, Multiboot2Header, RelocatableHeaderTag,
        RelocatableHeaderTagPreference,
    };

    #[test]
    fn test_size_or_up_aligned() {
        assert_eq!(0, HeaderBuilder::size_or_up_aligned(0));
        assert_eq!(8, HeaderBuilder::size_or_up_aligned(1));
        assert_eq!(8, HeaderBuilder::size_or_up_aligned(8));
        assert_eq!(16, HeaderBuilder::size_or_up_aligned(9));
    }

    #[test]
    fn test_builder() {
        // Step 1/2: Build Header
        let mb2_hdr_data = {
            let builder = HeaderBuilder::new(HeaderTagISA::I386);
            // Multiboot2 basic header + end tag
            let mut expected_len = 16 + 8;
            assert_eq!(builder.expected_len(), expected_len);

            // add information request tag
            let ifr_builder = InformationRequestHeaderTagBuilder::new(HeaderTagFlag::Required)
                .add_irs(&[
                    MbiTagType::EfiMmap,
                    MbiTagType::Cmdline,
                    MbiTagType::ElfSections,
                ]);
            let ifr_tag_size_with_padding = ifr_builder.expected_len() + 4;
            assert_eq!(
                ifr_tag_size_with_padding % 8,
                0,
                "the length of the IFR tag with padding must be a multiple of 8"
            );
            expected_len += ifr_tag_size_with_padding;
            let builder = builder.information_request_tag(ifr_builder);
            assert_eq!(builder.expected_len(), expected_len);

            let builder = builder.relocatable_tag(RelocatableHeaderTag::new(
                HeaderTagFlag::Required,
                0x1337,
                0xdeadbeef,
                4096,
                RelocatableHeaderTagPreference::None,
            ));
            expected_len += 0x18;
            assert_eq!(builder.expected_len(), expected_len);

            println!("builder: {:#?}", builder);
            println!("expected_len: {} bytes", builder.expected_len());

            builder.build()
        };

        // Step 2/2: Test the built Header
        {
            let mb2_hdr = mb2_hdr_data.as_ptr().cast();
            let mb2_hdr = unsafe { Multiboot2Header::load(mb2_hdr) }
                .expect("the generated header to be loadable");
            println!("{:#?}", mb2_hdr);
            assert_eq!(
                mb2_hdr.relocatable_tag().unwrap().flags(),
                HeaderTagFlag::Required
            );
            assert_eq!(mb2_hdr.relocatable_tag().unwrap().min_addr(), 0x1337);
            assert_eq!(mb2_hdr.relocatable_tag().unwrap().max_addr(), 0xdeadbeef);
            assert_eq!(mb2_hdr.relocatable_tag().unwrap().align(), 4096);
            assert_eq!(
                mb2_hdr.relocatable_tag().unwrap().preference(),
                RelocatableHeaderTagPreference::None
            );

            /* you can write the binary to a file and a tool such as crate "bootinfo"
               will be able to fully parse the MB2 header
            let mut file = std::file::File::create("mb2_hdr.bin").unwrap();
            use std::io::Write;
            file.write_all(mb2_hdr_data.as_slice()).unwrap();*/
        }
    }
}
