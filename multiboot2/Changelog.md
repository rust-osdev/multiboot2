# CHANGELOG for crate `multiboot2`

## 0.12.2
- `TagType` now implements `Eq` and `Hash`
- internal improvements
  - `std` can be used in tests; the crate is still `no_std`
    - this implies that `cargo test` doesn't work on "non-standard" targets
    - CI (Ubuntu) still works.
  - code formatting/style
  - sensible style checks as optional CI job
  - `.editorconfig` file
  - prepared co-existence of crates `multiboot2` and `multiboot2-header`
    in a Cargo workspace inside the same repository

## 0.12.1
- `TagType`-enum introduced in `v0.11` is now actually public
- internal code improvements

## 0.12.0

- **breaking:** `load()` and `load_with_offset` now returns a result
- added public constant `MULTIBOOT2_BOOTLOADER_MAGIC`
- Rust edition 2018 (instead of 2015)
- internal code improvements

## 0.11.0

- lib now contains `TagType`-enum that contains
  all possible mbi tags that are specified (taken from spec)
- much improved debug-formatting of `BootInformation`
- internal code improvements / formatting

## 0.10.0
- allow access to all memory regions (MemoryMap-Tag)
- internal code improvements

## 0.9.0

- Add a `checksum_is_valid` method to the RSDP tags ([#64](https://github.com/rust-osdev/multiboot2/pull/64))

## 0.8.2

- Add some basic documentation ([#62](https://github.com/rust-osdev/multiboot2/pull/62))
- Add MemoryAreaType, to allow users to access memory area types in a type-safe way ([#61](https://github.com/rust-osdev/multiboot2/pull/61))
