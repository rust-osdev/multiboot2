# multiboot2-common

[![crates.io](https://img.shields.io/crates/v/multiboot2-common.svg)](https://crates.io/crates/multiboot2-common)
[![docs](https://docs.rs/multiboot2-common/badge.svg)](https://docs.rs/multiboot2-common/)

Common helpers for the `multiboot2` and `multiboot2-header` crates.

## Architecture

The following figures, not displayable in `lib.rs` / on `docs.rs` unfortunately,
outline the design of this crate:

![Overview Multiboot2 Structures](./overview-multiboot2-structures.drawio.png "Overview Multiboot2 Structures")

Overview of Multiboot2 structures: Multiboot2 boot information, boot
information tags, Multiboot2 headers, and Multiboot2 header tags all share the
same technical foundation: They have a common header and a possible dynamic
size, depending on the header.

![Crate Architecture Overview](./architecture.drawio.png "Crate Architecture Overview")

Overview of how raw bytes are modelled to be representable by high-level
ABI-compatible rusty types.

## MSRV

The MSRV is 1.75.0 stable.

## License & Contribution

See main [README](https://github.com/rust-osdev/multiboot2/blob/main/README.md)
file.
