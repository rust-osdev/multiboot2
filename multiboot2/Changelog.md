# CHANGELOG for crate `multiboot2`

## Unreleased
- MSRV is 1.56.1
- **BREAKING** fixed lifetime issues: `VBEInfoTag` is no longer static
- fixed another internal lifetime issue
- `BootInformation::framebuffer_tag()` now returns
  `Option<Result<FramebufferTag, UnknownFramebufferType>>` instead of
  `Option<FramebufferTag>` which prevents a possible panic. If the `--unstable`
  feature is used, `UnknownFramebufferType` implements `core::error::Error`.
- Fixed misleading documentation of the `BootInformation::efi_memory_map_tag`
  tag.

## 0.14.2 (2023-03-17)
- documentation fixes
- `MbiLoadError` now implements `Display`
- Added the `unstable` feature, which enables nightly-only functionality.
  With this feature, `MbiLoadError` now implements `core::error::Error` and can
  be used with `anyhow::Result` for example.

## 0.14.1 (2023-03-09)
- fixed the calculation of the last area of the memory map tag ([#119](https://github.com/rust-osdev/multiboot2/pull/119))
  (Previously, iterating the EFI Memory map resulted in a superfluous entry as it ran over the next tag)

## 0.14.0 (2022-06-30)
- **BREAKING CHANGES** \
  This version includes a few small breaking changes that brings more safety when parsing strings from the
  multiboot information structure.
  - `BootLoaderNameTag::name` now returns a Result instead of just the value
  - `CommandLineTag::command_line` now returns a Result instead of just the value
  - `ModuleTag::cmdline` now returns a Result instead of just the value
  - `RsdpV1Tag::signature` now returns a Result instead of an Option
  - `RsdpV1Tag::oem_id` now returns a Result instead of an Option
  - `RsdpV2Tag::signature` now returns a Result instead of an Option
  - `RsdpV2Tag::oem_id` now returns a Result instead of an Option
- internal code improvements

## 0.13.3 (2022-06-03)
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
