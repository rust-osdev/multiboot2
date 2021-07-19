extern crate multiboot2;

use multiboot2::{
    load_mb2_header, EfiBootServiceHeaderTag, EndHeaderTag, HeaderTagFlag, HeaderTagISA,
    InformationRequestHeaderTag, MbiTagType, Multiboot2HeaderBuilder,
};

/// Constructs a multiboot2 header on the fly during runtime.
fn main() {
    let mut mb2_header_tags_bytes = Vec::new();
    let mut mb2_header_bytes = Vec::new();

    add_bytes_8_byte_aligned(
        &mut mb2_header_tags_bytes,
        &get_bytes_of_structure(EfiBootServiceHeaderTag::new(HeaderTagFlag::Required)),
    );
    add_bytes_8_byte_aligned(
        &mut mb2_header_tags_bytes,
        &get_bytes_of_structure(InformationRequestHeaderTag::new(
            HeaderTagFlag::Required,
            [MbiTagType::AcpiV2, MbiTagType::Efi64Ih, MbiTagType::Efi64],
        )),
    );
    add_bytes_8_byte_aligned(
        &mut mb2_header_tags_bytes,
        &get_bytes_of_structure(EndHeaderTag::new()),
    );

    // static header begin
    let mb2_hdr_builder = Multiboot2HeaderBuilder::new()
        .arch(HeaderTagISA::I386)
        .additional_tags_len(mb2_header_tags_bytes.len() as u32);
    let mb2_hdr_bytes = mb2_hdr_builder.build();
    assert_eq!(mb2_hdr_bytes.len(), 4 * 4);
    mb2_header_bytes.extend_from_slice(&mb2_hdr_bytes);
    mb2_header_bytes.extend_from_slice(&mb2_header_tags_bytes);

    let data_ptr = mb2_header_bytes.as_ptr();
    // I rely on that Rust does this automatically on x86_64 machines
    assert_eq!(data_ptr as usize % 8, 0, "must be 8-byte aligned");

    /*for (i, byte) in mb2_header_bytes.iter().enumerate() {
        let ptr = unsafe { data_ptr.offset(i as isize) };
        println!("{:?}: {}", ptr, byte);
    }*/

    let mb2_header = unsafe { load_mb2_header(data_ptr as usize) };
    println!("{:#?}", mb2_header);
}

/// Transforms a Rust data structure to it's actual byte representation in memory.
fn get_bytes_of_structure<T>(t: T) -> Vec<u8> {
    let size = std::mem::size_of::<T>();
    let t_ptr = (&t) as *const T as *const u8;
    let mut vec = Vec::with_capacity(size);
    for i in 0..size {
        vec.push(unsafe { *t_ptr.offset(i as isize) })
    }
    vec
}

/// Add bytes to a vector. Makes sure it fills zeroes at the end so that
/// the next value could will be 8 byte aligned again.
fn add_bytes_8_byte_aligned(buf: &mut Vec<u8>, bytes: &[u8]) {
    // always make sure the vector itself is correctly aligned
    buf.extend_from_slice(bytes);
    let fill_zeroes = buf.len() % 8;
    for _ in 0..fill_zeroes {
        buf.push(0);
    }
}
