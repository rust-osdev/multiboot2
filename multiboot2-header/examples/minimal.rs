use multiboot2_header::builder::Multiboot2HeaderBuilder;
use multiboot2_header::{
    HeaderTagFlag, HeaderTagISA, InformationRequestHeaderTagBuilder, MbiTagType, Multiboot2Header,
    RelocatableHeaderTag, RelocatableHeaderTagPreference,
};

/// Small example that creates a Multiboot2 header and parses it afterwards.
fn main() {
    // We create a Multiboot2 header during runtime here. A practical example is that your
    // program gets the header from a file and parses it afterwards.
    let mb2_hdr_bytes = Multiboot2HeaderBuilder::new(HeaderTagISA::I386)
        .relocatable_tag(RelocatableHeaderTag::new(
            HeaderTagFlag::Required,
            0x1337,
            0xdeadbeef,
            4096,
            RelocatableHeaderTagPreference::None,
        ))
        .information_request_tag(
            InformationRequestHeaderTagBuilder::new(HeaderTagFlag::Required)
                .add_irs(&[MbiTagType::Cmdline, MbiTagType::BootLoaderName]),
        )
        .build();

    // Cast bytes in vector to Multiboot2 information structure
    let mb2_hdr = unsafe { Multiboot2Header::from_addr(mb2_hdr_bytes.as_ptr() as usize) };
    println!("{:#?}", mb2_hdr);
}
