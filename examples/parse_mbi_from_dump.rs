use std::fs::File;
use std::io::Read;

fn main() {
    // This dump comes from my QEMU setup where I built my own multiboot2 Rust kernel.
    // The MBI contains all information provided by GRUB, after a handoff in x86_64 state
    // with UEFI boot services enabled.
    //
    // It is good to quickly check the library, i.e. the debug trait or if other changes
    // still result in correct parsing of the MBI.
    let mut file = File::open("examples/mbi_dump_example.bin").unwrap();
    let mut vec = Vec::new();
    file.read_to_end(&mut vec).unwrap();
    let bi_ptr = vec.into_boxed_slice();
    let bi_ptr = bi_ptr.as_ptr() as u64;
    let bi = unsafe { multiboot2::load(bi_ptr as usize) };
    println!("{:#?}", bi);
}