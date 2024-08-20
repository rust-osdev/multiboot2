# CHANGELOG for crate `multiboot2-header`

## v0.5.0

This release contains a major refactoring of the internals, guaranteeing
even more sanity checks for correct behaviour and lack of UB. In this release,
the `Builder` was rewritten and lots of corresponding UB in certain
corer-cases removed. Further, the builder's API was streamlined.

If you are interested in the internals of the major refactorings recently taken
place, please head to the documentation of `multiboot2-common`.

- **Breaking** All functions that returns something useful are now `#[must_use]`
- **Breaking** The builder type is now just called `Builder`. This needs the
  `builder` feature.
- Updated to latest `multiboot2` dependency

## 0.4.0 (2024-05-01)

- added `EndHeaderTag::default()`
- MSRV is 1.70
- Can add multiple `TagType::Smbios` tags in the builder.

## 0.3.2 (2023-11-30)

- **BREAKING** bumped `multiboot2` dependency to `v0.19.0`
- the `multiboot2` dependency doesn't pull in the `multiboot2/builder` feature
  anymore
- doc update

## 0.3.1 (2023-06-28)

- doc update

## 0.3.0 (2023-06-23)

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

## 0.2.0 (2022-05-03)

- **BREAKING** renamed `EntryHeaderTag` to `EntryAddressHeaderTag`
- **BREAKING** some paths changed from `multiboot2_header::header`
  to `multiboot2_header::builder`
  -> thus, import paths are much more logically now
- internal code improvements

## 0.1.1 (2022-05-02)

- fixed a bug that prevented the usage of the crate in `no_std` environments
- added a new default `builder`-feature to Cargo which requires the `alloc`
  -crate
  (this feature can be disabled which will also remove the dependency to
  the `alloc` crate)

## 0.1.0 (2021-10-08)

- initial release

## 0.0.0

Empty release to save the name on crates.io
