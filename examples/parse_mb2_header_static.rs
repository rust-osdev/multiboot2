use std::fs::File;
use std::io::Read;

/// Reads a multiboot2 header that is stored in a as binary in a file.
fn main() {
    let mut bin = File::open("examples/mb2_header.bin").unwrap();
    let mut data = Vec::new();
    bin.read_to_end(&mut data).unwrap();
    let data_ptr = data.as_ptr();
    assert_eq!(
        data_ptr as usize % 8,
        0,
        "must be 8 byte aligned (only on 64 bit arch by default)"
    );
    println!(
        "aligned data is stored in memory at location: {:?}",
        data_ptr
    );

    /*for (i, byte) in mb2_header_bytes.iter().enumerate() {
        let ptr = unsafe { data_ptr.offset(i as isize) };
        println!("{:?}: {}", ptr, byte);
    }*/

    // println!("actual location: {:?}", data_ptr);
    let mb2_header = unsafe { multiboot2::tags::header::load_mb2_header(data_ptr as usize) };
    /*println!("0x{:x}", mb2_header.header_magic());
    println!("0x{:x}", mb2_header.length());*/
    println!("{:#?}", mb2_header);
}
