# multiboot2
![Build](https://github.com/rust-osdev/multiboot2/actions/workflows/rust.yml/badge.svg)
[![crates.io](https://img.shields.io/crates/v/multiboot2.svg)](https://crates.io/crates/multiboot2)
[![docs](https://docs.rs/multiboot2/badge.svg)](https://docs.rs/multiboot2/)

Rust library that helps you to parse the multiboot information structure (mbi) from
Multiboot2-compliant bootloaders, like GRUB. It supports all tags from the specification
including full support for the sections of ELF-64 files. This library is `no_std` and can be
used in a Multiboot2-kernel.

It follows the Multiboot 2.0 specification at https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html and the ELF 64 specification at http://www.uclibc.org/docs/elf-64-gen.pdf.

## Background: The Multiboot 2 Information Structure
The Multiboot information structure looks like this:

Field            | Type
---------------- | -----------
total size       | u32
reserved         | u32
tags             | variable
end tag = (0, 8) | (u32, u32)

There are many different types of tags, but they all have the same beginning:

Field         | Type
------------- | -----------------
type          | u32
size          | u32
other fields  | variable

All tags and the mbi itself are 8-byte aligned. The last tag must be the _end tag_, which is a tag of type `0` and size `8`.

## License & Contribution

See main [README](https://github.com/rust-osdev/multiboot2/blob/main/README.md) file.
