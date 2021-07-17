# 0.9.0 - 0.12.0
- the crate now publicly exports comprehensive type definitions for "multiboot2 header tags"
  and regular "multiboot2 tags" (inside "multiboot2 information structure" aka *MBI*)
- updated documentation. The header tags can be used to parse headers but should not be used
  to construct a multiboot2 header by yourself in a Rust application. This wouldn't work for 
  several reasons. This crate is great for parsing multiboot2 data structures, not for creating them.

# 0.9.0

- Add a `checksum_is_valid` method to the RSDP tags ([#64](https://github.com/rust-osdev/multiboot2-elf64/pull/64))

# 0.8.2

- Add some basic documentation ([#62](https://github.com/rust-osdev/multiboot2-elf64/pull/62))
- Add MemoryAreaType, to allow users to access memory area types in a type-safe way ([#61](https://github.com/rust-osdev/multiboot2-elf64/pull/61))
