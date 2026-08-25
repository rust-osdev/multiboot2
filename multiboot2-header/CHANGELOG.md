# Changelog for Crate `multiboot2-header`

## Unreleased

- **Breaking:** `InformationRequestHeaderTag::new()` now takes
  `&[MbiTagType]` and `requests()` returns an iterator over `MbiTagType`.
  The `MbiTagTypeId` re-export was renamed to `MbiTagTypeRaw`.
- Fixed undefined behavior when parsing a header containing a tag with a type
  unknown to the specification. `HeaderTagHeader` now stores the new
  `HeaderTagTypeRaw` newtype; the `typ()` getters keep returning
  `HeaderTagType`, which gained a `Custom` variant. **Breaking:** the now
  meaningless `HeaderTagType::count()` was removed.
- Fixed undefined behavior when parsing a header with an architecture unknown
  to the specification. `Multiboot2BasicHeader` now stores the new
  `HeaderTagISARaw` newtype; the `arch()` getters keep returning
  `HeaderTagISA`, which gained a `Custom` variant.
- Fixed undefined behavior when parsing a `RelocatableHeaderTag` with a
  placement preference unknown to the specification. The tag now stores the
  new `RelocatableHeaderTagPreferenceRaw` newtype; `preference()` keeps
  returning `RelocatableHeaderTagPreference`, which gained a `Custom` variant.
- Fixed undefined behavior when parsing a header tag whose flags field holds a
  value other than 0 or 1. `HeaderTagHeader` now stores the new
  `HeaderTagFlagRaw` newtype; the `flags()` getters keep returning
  `HeaderTagFlag`, which gained a `Custom` variant.
- Fixed undefined behavior and a spec violation in `ConsoleHeaderTagFlags`:
  the enum discriminants did not match the specification. **Breaking:**
  `ConsoleRequired` now serializes to `1` (was `0`) and `EgaTextSupported` to
  `2` (was `1`), matching the example C code of the specification; the enum
  gained a `Custom` variant and the tag stores the new
  `ConsoleHeaderTagFlagsRaw` newtype.

## v0.10.0 (2026-08-24)

- Fixed `Header::load` rejecting a valid header whose end tag has a non-zero
  flags field. The spec only requires the end tag to have type 0 and size 8, so
  the flags field is no longer constrained.

## v0.9.0 (2026-08-13)

- Expanded `Header` debug output with parsed tags and tag headers.
- **Breaking:** Renamed `multiboot2_header::Multiboot2Header` to
  `multiboot2_header::Header`.
- Standardized bootloader terminology.
- Corrected the README example and API documentation.

## v0.8.0 (2026-06-23)

- Fixed `Multiboot2Header::load` to validate the complete padded tag sequence.
- Changed `Multiboot2Header::find_header` to scan the full 32 KiB search window,
  validate candidate headers, and return the parsed header plus offset.
- Added validation that loaded headers end with the mandatory end tag.
- Changed checksum validation errors to include the actual and expected checksum
  values.
- Fixed `EndHeaderTag::new` and `Builder::build` so generated headers contain
  the mandatory end tag.
- Small code improvements

## v0.7.0 (2025-06-01)

- **Breaking:** Removed the optional `unstable` feature (required nightly)
  - `core::error::Error` is now implemented unconditionally
- **Breaking:** The MSRV is now 1.85

## v0.6.0 (2024-09-17)

- dependency updates
- **Breaking:** MSRV is now 1.75
- misc metadata fixes

## v0.5.1 (2024-08-24)

- Documentation improvements

## v0.5.0 (2024-05-20)

This release contains a major refactoring of the internals, guaranteeing even
more sanity checks for correct behaviour and lack of UB. In this release, the
`Builder` was rewritten and lots of corresponding UB in certain corner cases
removed. Further, the builder's API was streamlined.

If you are interested in the internals of the major refactorings recently taken
place, please head to the documentation of `multiboot2-common`.

- **Breaking** All functions that returns something useful are now `#[must_use]`
- **Breaking** Renamed `multiboot2_header::builder::HeaderBuilder` to
  `multiboot2_header::Builder`. This needs the `builder` feature.
- **Breaking:** The error type returned by `Multiboot2Header::load` has been
  changed.
- Updated to latest `multiboot2` dependency

All previous versions have been marked as **YANKED**. `0.5.0` is the first
version where all unit tests are passed by Miri, i.e., the first version without
Undefined Behavior.

## 0.4.0 (2024-05-01) (**YANKED**)

- added `EndHeaderTag::default()`
- MSRV is 1.70
- Can add multiple `TagType::Smbios` tags in the builder.

## 0.3.2 (2023-11-30) (**YANKED**)

- **BREAKING** bumped `multiboot2` dependency to `v0.19.0`
- the `multiboot2` dependency doesn't pull in the `multiboot2/builder` feature
  anymore
- doc update

## 0.3.1 (2023-06-28) (**YANKED**)

- doc update

## 0.3.0 (2023-06-23) (**YANKED**)

- **BREAKING** MSRV is 1.68.0 (UPDATE: This is actually 1.69.)
- **BREAKING** renamed the `std` feature to `alloc`
- **BREAKING** bumped `multiboot2` dependency to `v0.16.0`
- **BREAKING** renamed `MULTIBOOT2_HEADER_MAGIC` to `MAGIC`
- **BREAKING** renamed `Multiboot2HeaderBuilder` to `HeaderBuilder`
- **BREAKING** renamed `from_addr` to `load`. The function now consumes a ptr.
- **BREAKING** `HeaderBuilder::build` now returns a value of type `HeaderBytes`
  The old builder could produce misaligned structures.
- added the optional `unstable` feature (requires nightly)
  - implement `core::error::Error` for `LoadError`

## 0.2.0 (2022-05-03) (**YANKED**)

- **BREAKING** renamed `EntryHeaderTag` to `EntryAddressHeaderTag`
- **BREAKING** some paths changed from `multiboot2_header::header`
  to `multiboot2_header::builder`
  -> thus, import paths are much more logically now
- internal code improvements

## 0.1.1 (2022-05-02) (**YANKED**)

- fixed a bug that prevented the usage of the crate in `no_std` environments
- added a new default `builder`-feature to Cargo which requires the `alloc`
  -crate (this feature can be disabled which will also remove the dependency to
  the `alloc` crate)

## 0.1.0 (2021-10-08) (**YANKED**)

- initial release

## 0.0.0

Empty release to save the name on crates.io
