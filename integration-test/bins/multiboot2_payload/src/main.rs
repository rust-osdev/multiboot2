#![no_main]
#![no_std]

extern crate alloc;

#[macro_use]
extern crate util;

core::arch::global_asm!(include_str!("start.S"), options(att_syntax));
core::arch::global_asm!(include_str!("multiboot2_header.S"));

use multiboot2::BootInformation;
use util::{init_environment, qemu_exit_success};

mod verify;

/// Entry into the Rust code from assembly.
#[unsafe(no_mangle)]
fn rust_entry(multiboot2_magic: u32, multiboot2_hdr: u32) -> ! {
    main(multiboot2_magic, multiboot2_hdr).expect("Should run multiboot2 integration test");
    log::info!("Integration test finished successfully");
    qemu_exit_success()
}

/// Executes the main logic.
fn main(multiboot2_magic: u32, multiboot2_hdr: u32) -> anyhow::Result<()> {
    init_environment();
    if multiboot2_magic != multiboot2::MAGIC {
        Err(anyhow::Error::msg("Invalid bootloader magic"))?
    }
    log::debug!(
        "multiboot2_hdr=0x{multiboot2_hdr:08x?}, multiboot2_magic=0x{multiboot2_magic:08x?}"
    );

    let mbi_ptr = (multiboot2_hdr as *const u8).cast();
    let mbi = unsafe { BootInformation::load(mbi_ptr) }.map_err(anyhow::Error::msg)?;
    verify::run(&mbi)
}
