//! Common helpers for the `multiboot2` and `multiboot2-header` crates.
//!
//! The main objective here is to encapsulate the memory-sensible part of
//! parsing and iterating Multiboot2 structures. This crate empowers
//! `multiboot2` and `multiboot2-header` to use rusty types for the
//! corresponding structures, such as non-trait DSTs (structs with a
//! terminating `[u8]` field). The abstractions can be used for:
//! - multiboot2:
//!   - boot information structure (whole)
//!   - boot information tags
//! - multiboot2-header:
//!   - header structure (whole)
//!   - header tags
//!
//! Not meant as stable public API for others.

#![no_std]
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
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(rustdoc::all)]
// --- END STYLE CHECKS ---
