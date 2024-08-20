//! Rust library with type definitions and parsing functions for Multiboot2
//! headers, as well as a builder to build them at runtime. This library is
//! `no_std` and can be used in bootloaders.
//!
//! # Example: Parsing a Header
//!
//! ```no_run
//! use multiboot2_header::Multiboot2Header;
//!
//! let ptr = 0x1337_0000 as *const u8 /* use real ptr here */;
//! let mb2_hdr = unsafe { Multiboot2Header::load(ptr.cast()) }.unwrap();
//! for _tag in mb2_hdr.iter() {
//!     //
//! }
//! ```
//!
//! ## MSRV
//!
//! The MSRV is 1.70.0 stable.

#![no_std]
#![cfg_attr(feature = "unstable", feature(error_in_core))]
// --- BEGIN STYLE CHECKS ---
#![deny(
    clippy::all,
    clippy::cargo,
    clippy::nursery,
    clippy::must_use_candidate,
    // clippy::restriction,
    // clippy::pedantic
)]
// now allow a few rules which are denied by the above statement
// --> They are either ridiculous, not necessary, or we can't fix them.
#![allow(clippy::multiple_crate_versions)]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(rustdoc::all)]
// --- END STYLE CHECKS ---

#[cfg(feature = "builder")]
extern crate alloc;

#[cfg_attr(test, macro_use)]
#[cfg(test)]
extern crate std;

/// Iterator over the tags of a Multiboot2 boot information.
pub type TagIter<'a> = multiboot2_common::TagIter<'a, HeaderTagHeader>;

/// A generic version of all boot information tags.
#[cfg(test)]
pub type GenericHeaderTag = multiboot2_common::DynSizedStructure<HeaderTagHeader>;

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
mod builder;

pub use multiboot2_common::{DynSizedStructure, MaybeDynSized, Tag};

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
#[cfg(feature = "builder")]
pub use builder::Builder;

/// Re-export of [`multiboot2::TagType`] from `multiboot2`-crate.
pub use multiboot2::{TagType as MbiTagType, TagTypeId as MbiTagTypeId};
