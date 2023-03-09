//! Library with type definitions and parsing functions for Multiboot2 headers.
//! This library is `no_std` and can be used in bootloaders.
//!
//! # Example
//! ```rust
//! use multiboot2_header::builder::{InformationRequestHeaderTagBuilder, Multiboot2HeaderBuilder};
//! use multiboot2_header::{HeaderTagFlag, HeaderTagISA, MbiTagType, RelocatableHeaderTag, RelocatableHeaderTagPreference, Multiboot2Header};
//!
//! // Small example that creates a Multiboot2 header and parses it afterwards.
//!
//! // We create a Multiboot2 header during runtime here. A practical example is that your
//! // program gets the header from a file and parses it afterwards.
//! let mb2_hdr_bytes = Multiboot2HeaderBuilder::new(HeaderTagISA::I386)
//!     .relocatable_tag(RelocatableHeaderTag::new(
//!         HeaderTagFlag::Required,
//!         0x1337,
//!         0xdeadbeef,
//!         4096,
//!         RelocatableHeaderTagPreference::None,
//!     ))
//!     .information_request_tag(
//!         InformationRequestHeaderTagBuilder::new(HeaderTagFlag::Required)
//!             .add_irs(&[MbiTagType::Cmdline, MbiTagType::BootLoaderName]),
//!     )
//!     .build();
//!
//! // Cast bytes in vector to Multiboot2 information structure
//! let mb2_hdr = unsafe { Multiboot2Header::from_addr(mb2_hdr_bytes.as_ptr() as usize) };
//! println!("{:#?}", mb2_hdr);
//!
//! ```
//!
//! ## MSRV
//! The MSRV is 1.60.0 stable.

#![no_std]
#![deny(rustdoc::all)]
#![allow(rustdoc::missing_doc_code_examples)]
#![deny(clippy::all)]
#![deny(clippy::missing_const_for_fn)]
#![deny(missing_debug_implementations)]

#[cfg(feature = "builder")]
extern crate alloc;

#[cfg_attr(test, macro_use)]
#[cfg(test)]
extern crate std;

mod address;
mod console;
mod end;
mod entry_address;
mod entry_efi_32;
mod entry_efi_64;
mod framebuffer;
mod header;
mod information_request;
mod module_align;
mod relocatable;
mod tags;
mod uefi_bs;

#[cfg(feature = "builder")]
pub mod builder;

pub use self::address::*;
pub use self::console::*;
pub use self::end::*;
pub use self::entry_address::*;
pub use self::entry_efi_32::*;
pub use self::entry_efi_64::*;
pub use self::framebuffer::*;
pub use self::header::*;
pub use self::information_request::*;
pub use self::module_align::*;
pub use self::relocatable::*;
pub use self::tags::*;
pub use self::uefi_bs::*;

/// Re-export of [`multiboot2::TagType`] from `multiboot2`-crate as `MbiTagType`, i.e. tags that
/// describe the entries in the Multiboot2 Information Structure (MBI).
pub use multiboot2::TagType as MbiTagType;
