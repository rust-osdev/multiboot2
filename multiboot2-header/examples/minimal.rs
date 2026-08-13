use multiboot2_header::{
    Builder, HeaderTagFlag, HeaderTagISA, InformationRequestHeaderTag, MaybeDynSized, MbiTagType,
    Multiboot2Header, RelocatableHeaderTag, RelocatableHeaderTagPreference,
};

/// Small example that creates a Multiboot2 header and parses it afterwards.
fn main() {
    // We create a Multiboot2 header during runtime here. A more practical
    // example, however, would be that you parse the header from kernel binary
    // at runtime.
    let header_bytes = Builder::new(HeaderTagISA::I386)
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

    let header = unsafe { Multiboot2Header::load(header_bytes.as_ptr()) }.unwrap();
    println!("{header:#?}");
}
