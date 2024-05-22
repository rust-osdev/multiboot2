#![no_main]
#![no_std]
#![feature(error_in_core)]

mod loader;
mod multiboot;

extern crate alloc;

#[macro_use]
extern crate util;

use util::init_environment;

core::arch::global_asm!(include_str!("start.S"), options(att_syntax));

/// Entry into the Rust code from assembly using the x86 SystemV calling
/// convention.
#[no_mangle]
fn rust_entry(multiboot_magic: u32, multiboot_hdr: *const u32) -> ! {
    init_environment();
    log::debug!("multiboot_hdr={multiboot_hdr:x?}, multiboot_magic=0x{multiboot_magic:x?}");
    let mbi = multiboot::get_mbi(multiboot_magic, multiboot_hdr as u32).unwrap();
    let module_iter = mbi.modules().expect("Should provide modules");
    loader::load_module(module_iter);
}
