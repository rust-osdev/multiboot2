use core::mem::size_of;
use ::{HeaderTagISA, MULTIBOOT2_HEADER_MAGIC};
use alloc::vec::Vec;
use Multiboot2HeaderInner;

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
