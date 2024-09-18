#![no_main]
#![no_std]
#![feature(error_in_core)]

mod loader;

extern crate alloc;

#[macro_use]
extern crate util;

use util::init_environment;

core::arch::global_asm!(include_str!("multiboot2_header.S"), options(att_syntax));
core::arch::global_asm!(include_str!("start.S"), options(att_syntax));

/// Entry into the Rust code from assembly using the x86 SystemV calling
/// convention.
#[no_mangle]
fn rust_entry(multiboot_magic: u32, multiboot_hdr: *const u32) -> ! {
    init_environment();
    log::debug!("multiboot_hdr={multiboot_hdr:x?}, multiboot_magic=0x{multiboot_magic:x?}");
    assert_eq!(multiboot_magic, multiboot2::MAGIC);
    let mbi = unsafe { multiboot2::BootInformation::load(multiboot_hdr.cast()) }.unwrap();

    if let Some(mmap) = mbi.efi_memory_map_tag() {
        log::debug!("efi memory map:",);
        for desc in mmap.memory_areas() {
            log::warn!(
                "  start=0x{:016x?} size={:016x?} type={:?}, attr={:?}",
                desc.phys_start,
                desc.page_count * 4096,
                desc.ty,
                desc.att
            );
        }
    }

    loader::load_module(&mbi);
}
