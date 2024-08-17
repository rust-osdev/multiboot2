# CHANGELOG for crate `multiboot2`

## Unreleased

-

## 0.21.0 (2024-08-17)

This release contains a massive refactoring of various internals. Now, **all
unit tests pass Miri**, thus we removed lots of undefined behaviour and
increased the memory safety! 🎉 Only a small part of these internal refactorings
leak to the public interface. If you don't provide external custom tags, you
should be fine.

Please note that **all previous releases** must be considered unsafe, as they
contain UB. However, it is never clear how UB results in immediate incorrect
behaviour and it _might_ work. **Nevertheless, please migrate to the latest
release and you'll be fine!**

All previous releases on crates.io have been yanked.

- **Breaking:** All functions that returns something useful are
  now `#[must_use]`
- **Breaking:** More public fields in tags were replaced by public getters, such
  as `SmbiosTag::major()`
- **Breaking:** Methods of `InformationBuilder` to add tags now consume
  references instead of owned values
- **Breaking:** The `BoxedDst` has been removed in favor of a normal Rust `Box`.
  This only affects you if you use the `builder` feature.
- **Breaking:** MSRV is 1.75
- **Breaking:** Introduced new `TagHeader` type as replacement for the `Tag`
  type that will be changed in the next step. `Tag` has been renamed to an
  internal-only `GenericTag` type.
- Added missing `InformationBuilder::vbe_info_tag`
- documentation enhancements
- updated dependencies

## 0.20.2 (2024-05-26)

- fix Debug implementation of `EfiMemoryMapTag`

## 0.20.1 (2024-05-26)

- fixed the handling of `EFIMemoryMapTag` and `EFIMemoryAreaIter`
- **BREAKING** Fixed wrong creation of `EFIMemoryMapTag`.
  `EFIMemoryMapTag::new` was replaced by `EFIMemoryMapTag::new_from_descs` and
  `EFIMemoryMapTag::new_from_map`.
- `ModuleTag::new`'s `end` parameter now must be bigger than `start`.

## 0.20.0 (2024-05-01)

- added `InformationBuilder::default()`
- MSRV is 1.70

## 0.19.0 (2023-09-21)

- **BREAKING** MSRV is 1.69.0
- **BREAKING** `Tag::get_dst_str_slice` renamed to
  `Tag::parse_slice_as_string` and now returns `Result<&str, StringError>`
- **BREAKING** `BootLoaderNameTag::name` now returns `Result<&str, StringError>`
- **BREAKING** `CommandLineTag::cmdline` now returns `Result<&str, StringError>`
- **BREAKING** `ModuleTag::cmdline` now returns `Result<&str, StringError>`
- Introduced new enum type `StringError`
- Additionally, a bug was fixed in `parse_slice_as_string` which now parses
  multiboot2 strings as expected: as null-terminated UTF-8 strings.
- `InformationBuilder` now also allows to add custom tags. The new public method
  `add_tag` was introduced for that.

## 0.18.1 (2023-07-13)

- Documentation improvements

## 0.18.0 (2023-07-13)

- **BREAKING** The `TagTrait` was enhanced and now has an associated `ID`
  constant. This is only breaking to users that used `BootInformation::get_tag`
  or that implement custom tags. `BootInformation::get_tag` doesn't need the
  `typ` parameter anymore, as it can be deduced from the provided type.
- **BREAKING** `BoxedDst::new` doesn't have the `typ` parameter anymore. This
  only effects you when you wrote a custom DST tag.
- **BREAKING** Removed deprecated functions `load` and `load_with_offset`. Use
  `BootInformation::load` instead.
- **BREAKING** Renamed `BootInformation::efi_32_ih_tag` to
  `BootInformation::efi_ih32_tag` for consistency.
- **BREAKING** Renamed `BootInformation::efi_64_ih_tag` to
  `BootInformation::efi_ih64_tag` for consistency.
- **BREAKING** Renamed `BootInformation::efi_std_32_tag` to
  `BootInformation::efi_std32_tag` for consistency.
- **BREAKING** Renamed `BootInformation::efi_std_64_tag` to
  `BootInformation::efi_std64_tag` for consistency.
- Better debug output of `BootInformation` and `MemoryArea`
- Internal code cleanup.

## 0.17.0 (2023-07-12)

- **BREAKING** Make functions of `InformationBuilder` chainable. They now
  consume the builder.
- **BREAKING** Allow non-standard memory area types by using new pair of
  corresponding types: `MemoryAreaTypeId` and `MemoryAreaType`.

## 0.16.0 (2023-06-23)

- **BREAKING** renamed `MULTIBOOT2_BOOTLOADER_MAGIC` to `MAGIC`
- **BREAKING** `EFIMemoryDesc` was removed and is now an alias of
  `uefi_raw::table::boot::MemoryDescriptor`
- **BREAKING**  `EFIMemoryAreaType` was removed and is now an alias of
  `uefi_raw::table::boot::MemoryType`
- **BREAKING** MSRV is 1.68.0
- **BREAKING** Removed `MemoryAreaIter`
  and `MemoryMapTag::available_memory_areas`
- **BREAKING** Renamed `BootInformation::load_base_addr`
  to `BootInformation::load_base_addr_tag`
- **BREAKING** Renamed `BootInformation::efi_32_ih`
  to `BootInformation::efi_32_ih_tag`
- **BREAKING** Renamed `BootInformation::efi_32_ih`
  to `BootInformation::efi_32_ih_tag`
- **BREAKING** Renamed `ImageLoadPhysAddr` to `ImageLoadPhysAddrTag`
- **BREAKING** Renamed `EFIImageHandle32` to `EFIImageHandle32Tag`
- **BREAKING** Renamed `EFIImageHandle64` to `EFIImageHandle64Tag`
- **BREAKING** Renamed `EFISdt32` to `EFISdt32Tag`
- **BREAKING** Renamed `EFISdt64` to `EFISdt64Tag`
- **BREAKING** Renamed `EFIBootServicesNotExited`
  to `EFIBootServicesNotExitedTag`
- **BREAKING** Renamed `CommandLineTag::command_line` renamed
  to `CommandLineTag::cmdline`
- **\[Might be\] BREAKING** Added `TagTrait` trait which enables to use DSTs as
  multiboot2 tags. This is
  mostly relevant for the command line tag, the modules tag, and the bootloader
  name tag. However, this might also be relevant for users of custom multiboot2
  tags that use DSTs as types. See the example provided in the doc of the
  `get_tag` method.
- added a `builder` feature and a `builder` module with a
  `builder::InformationBuilder` struct
- added `BootInformation::efi_bs_not_exited_tag`
- deprecated `load` and `load_with_offset`
- added `BootInformation::load` as new default constructor
- added `MemoryMapTag::entry_size` and `MemoryMapTag::entry_version`

## 0.15.1 (2023-03-18)

- **BREAKING** `MemoryMapTag::all_memory_areas()` was renamed to `memory_areas`
  and now returns `MemoryAreaIter` instead of
  `impl Iterator<Item = &MemoryArea>`. Experience showed that its better to
  return the specific iterator whenever possible.
- **BREAKING** `MemoryMapTag::memory_areas()` was renamed to
  `available_memory_areas`
  (_Sorry for the breaking changes in a minor release, but I just stumbled upon
  this und since the last breaking release was just yesterday, users have to
  deal with changes anyway._)
- **BREAKING** `ElfSection::name()` now returns a Result instead of just the
  value. This prevents possible panics.
- fix: prevent a possible panic in `ElfSection::section_type()`

## 0.15.0 (2023-03-17)

- **BREAKING** MSRV is 1.56.1
- **BREAKING** fixed lifetime issues: `VBEInfoTag` is no longer `&static`
- **BREAKING:** `TagType` is now split into `TagTypeId` and `TagType`
    - `TagTypeId` is a binary-compatible form of a Multiboot2 tag id
    - `TagType` is a higher-level abstraction for either specified or custom
      tags
      but not ABI compatible.
    - There exists a seamless integration between `u32`, `TagType`, and
      `TagTypeId` via `From` and `PartialEq`-implementations.
- fixed another internal lifetime issue
- `BootInformation::framebuffer_tag()` now returns
  `Option<Result<FramebufferTag, UnknownFramebufferType>>` instead of
  `Option<FramebufferTag>` which prevents a possible panic. If the `--unstable`
  feature is used, `UnknownFramebufferType` implements `core::error::Error`.
- Fixed misleading documentation of the `BootInformation::efi_memory_map_tag`
- `BootInformation` now publicly exports the `get_tag` function allowing you to
  work with custom tags. An example is given in the function documentation.
  (check docs.rs). There is also a small unit test that you can use to learn
  from.

## 0.14.2 (2023-03-17)

- documentation fixes
- `MbiLoadError` now implements `Display`
- Added the `unstable` feature, which enables nightly-only functionality.
  With this feature, `MbiLoadError` now implements `core::error::Error` and can
  be used with `anyhow::Result` for example.

## 0.14.1 (2023-03-09)

- fixed the calculation of the last area of the memory map
  tag ([#119](https://github.com/rust-osdev/multiboot2/pull/119))
  (Previously, iterating the EFI Memory map resulted in a superfluous entry as
  it ran over the next tag)

## 0.14.0 (2022-06-30)

- **BREAKING CHANGES** \
  This version includes a few small breaking changes that brings more safety
  when parsing strings from the
  multiboot information structure.
    - `BootLoaderNameTag::name` now returns a Result instead of just the value
    - `CommandLineTag::command_line` now returns a Result instead of just the
      value
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
- small internal improvements and restructuring of the code (no breaking changes
  to public API)

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

- **breaking:** iterator functions (e.g. `ElfSectionsTag::sections()`)
  return `impl Iterator` instead of a concrete type
- lib now contains `TagType`-enum that contains
  all possible mbi tags that are specified (taken from spec)
- much improved debug-formatting of `BootInformation`
- internal code improvements / formatting

## 0.10.0 (2020-11-03)

- allow access to all memory regions (MemoryMap-Tag)
- internal code improvements

## 0.9.0 (2020-07-06)

- Add a `checksum_is_valid` method to the RSDP
  tags ([#64](https://github.com/rust-osdev/multiboot2/pull/64))

## 0.8.2 (2022-03-02)

- Add some basic
  documentation ([#62](https://github.com/rust-osdev/multiboot2/pull/62))
- Add MemoryAreaType, to allow users to access memory area types in a type-safe
  way ([#61](https://github.com/rust-osdev/multiboot2/pull/61))
