use multiboot2_common::MaybeDynSized;
use multiboot2_header::Builder;
use multiboot2_header::{
    HeaderTagFlag, HeaderTagISA, InformationRequestHeaderTag, MbiTagType, Multiboot2Header,
    RelocatableHeaderTag, RelocatableHeaderTagPreference,
};

/// Small example that creates a Multiboot2 header and parses it afterwards.
fn main() {
    // We create a Multiboot2 header during runtime here. A more practical
    // example, however, would be that you parse the header from kernel binary
    // at runtime.
    let mb2_hdr_bytes = Builder::new(HeaderTagISA::I386)
        .relocatable_tag(RelocatableHeaderTag::new(
            HeaderTagFlag::Required,
            0x1337,
            0xdeadbeef,
            4096,
            RelocatableHeaderTagPreference::None,
        ))
        .information_request_tag(InformationRequestHeaderTag::new(
            HeaderTagFlag::Required,
            &[
                MbiTagType::Cmdline.into(),
                MbiTagType::BootLoaderName.into(),
            ],
        ))
        .build();

    // Cast bytes in vector to Multiboot2 information structure
    let ptr = mb2_hdr_bytes.as_bytes().as_ptr();
    let mb2_hdr = unsafe { Multiboot2Header::load(ptr.cast()) };
    let mb2_hdr = mb2_hdr.unwrap();
    println!("{:#?}", mb2_hdr);
}
