# CHANGELOG for crate `multiboot2`

## 0.13.3 (2022-05-03)
- impl `Send` for `BootInformation`

## 0.13.2 (2022-05-02)
- `TagType` now implements `Ord` so that it can be used in `BTreeSet`
- small internal improvements and restructuring of the code (no breaking changes to public API)

## 0.13.1 (2022-01-09)
- minor fix

## 0.13.0 (**yanked**)
- added missing getters for tag `ImageLoadPhysAddr`
- added missing getters for tags `EFIImageHandle32` and `EFIImageHandle64`

## 0.12.2 (2021-10-02)
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

## 0.12.1 (2021-08-11)
- `TagType`-enum introduced in `v0.11` is now actually public
- internal code improvements

## 0.12.0 (2021-08-06)
- **breaking:** `load()` and `load_with_offset` now returns a result
- added public constant `MULTIBOOT2_BOOTLOADER_MAGIC`
- Rust edition 2018 (instead of 2015)
- internal code improvements

## 0.11.0 (2021-07-07)

- **breaking:** iterator functions (e.g. `ElfSectionsTag::sections()`) return `impl Iterator` instead of a concrete type
- lib now contains `TagType`-enum that contains
  all possible mbi tags that are specified (taken from spec)
- much improved debug-formatting of `BootInformation`
- internal code improvements / formatting

## 0.10.0 (2020-11-03)
- allow access to all memory regions (MemoryMap-Tag)
- internal code improvements

## 0.9.0 (2020-07-06)

- Add a `checksum_is_valid` method to the RSDP tags ([#64](https://github.com/rust-osdev/multiboot2/pull/64))

## 0.8.2 (2022-03-02)

- Add some basic documentation ([#62](https://github.com/rust-osdev/multiboot2/pull/62))
- Add MemoryAreaType, to allow users to access memory area types in a type-safe way ([#61](https://github.com/rust-osdev/multiboot2/pull/61))
