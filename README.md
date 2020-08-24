# multiboot2-elf64
[![Build Status](https://travis-ci.org/rust-osdev/multiboot2-elf64.svg?branch=master)](https://travis-ci.org/rust-osdev/multiboot2-elf64)
[![crates.io](https://img.shields.io/crates/v/multiboot2.svg)](https://crates.io/crates/multiboot2)
[![docs](https://docs.rs/multiboot2/badge.svg)](https://docs.rs/multiboot2/)

An experimental Multiboot 2 crate for ELF-64 kernels. It's still incomplete, so please open an issue if you're missing some functionality. Contributions welcome!

It uses the Multiboot 2.0 specification at https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html and the ELF 64 specification at http://www.uclibc.org/docs/elf-64-gen.pdf.

Below is the draft for a blog post about this. I don't plan to finish it but maybe it's helpful as documentation.

## The Multiboot 2 Information Structure
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

All tags are 8-byte aligned. The last tag must be the _end tag_, which is a tag of type `0` and size `8`.

## Tags

We are interested in two tags, the _Elf-symbols_ tag and the _memory map_ tag. For a full list of possible tags see section 3.4 in the Multiboot 2 specification ([PDF][Multiboot 2]).

[Multiboot 2]: http://nongnu.askapache.com/grub/phcoder/multiboot.pdf

### The Elf-Symbols Tag
The Elf-symbols tag contains a list of all sections of the loaded [ELF] kernel. It has the following format:

[ELF]: http://www.uclibc.org/docs/elf-64-gen.pdf

Field                       | Type
--------------------------- | -----------------
type = 9                    | u32
size                        | u32
number of entries           | u32
entry size                  | u32
string table                | u32
section headers             | variable

Note that this format differs from the description in the Multiboot specification because it seems to be wrong for ELF 64 kernels: The `number of entries`, `entry size`, and `string table` fields seem to be `u32` instead of `u16`. The `multiboot2.h` file in the example section of the specification also specifies these fields as being `u32`, which suggests that the `u16` fields are an editing error. The GRUB2 bootloader [uses u32 fields](https://github.com/josefbacik/grub2/blob/96695ad84ce9c93f057ba53ae77d04d8561586e9/include/multiboot2.h#L298-L300), too.

The section headers are just copied from the ELF file, so we need to look at the ELF specification to find the corresponding structure definition. Our kernel is a 64-bit ELF file, so we need to look at the ELF-64 specification ([PDF][ELF specification]). According to section 4 and figure 3, a section header has the following format:

[ELF specification]: http://www.uclibc.org/docs/elf-64-gen.pdf

Field                       | Type             | Value
--------------------------- | ---------------- | -----------
name                        | u32              | string table index
type                        | u32              | `0` (unused), `1` (section of program), `3` (string table), `8` (uninitialized section), etc.
flags                       | u64              | `0x1` (writable), `0x2` (loaded), `0x4` (executable), etc.
address                     | u64              | virtual start address of section (0 if not loaded)
file offset                 | u64              | offset (in bytes) of section contents in the file
size                        | u64              | size of the section in bytes
link                        | u32              | associated section (only for some section types)
info                        | u32              | extra information (only for some section types)
address align               | u64              | required alignment of section (power of 2)
entry size                  | u64              | contains the entry size for table sections (e.g. string table)

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
