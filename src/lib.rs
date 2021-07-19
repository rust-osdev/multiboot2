#![no_std]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]

//! An experimental Multiboot 2 crate for ELF-64/32 kernels.
//!
//! The GNU Multiboot specification aims provide to a standardised
//! method of sharing commonly used information about the host machine at
//! boot time.

#[macro_use]
extern crate bitflags;
extern crate alloc;

mod header;
mod mbi;

pub use header::*;
pub use mbi::*;
