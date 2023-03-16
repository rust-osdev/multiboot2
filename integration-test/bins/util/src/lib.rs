#![no_std]

#[macro_use]
extern crate alloc;

use core::panic::PanicInfo;
use log::error;
use qemu_exit::QEMUExit;

static QEMU_EXIT: qemu_exit::X86 = qemu_exit::X86::new(QEMU_EXIT_PORT, QEMU_EXIT_SUCCESS);

#[macro_use]
pub mod macros;
pub mod allocator;
pub mod debugcon;

const QEMU_EXIT_PORT: u16 = 0xf4;
/// Custom error code to report success.
const QEMU_EXIT_SUCCESS: u32 = 73;

/// Initializes the environment.
pub fn init_environment() {
    debugcon::DebugconLogger::init();
    log::info!("Logger initialized!");
    allocator::init();
    log::info!("Allocator initialized! {:?}", vec![1, 2, 3]);
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    error!("PANIC! {}", info);
    qemu_exit_failure()
}

pub fn qemu_exit_success() -> ! {
    QEMU_EXIT.exit_success()
}

pub fn qemu_exit_failure() -> ! {
    QEMU_EXIT.exit_failure()
}
