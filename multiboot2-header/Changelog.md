# CHANGELOG for crate `multiboot2-header`

## Unreleased
- MSRV is 1.56.1

## v0.2.0 (2022-05-03)
- **BREAKING** renamed `EntryHeaderTag` to `EntryAddressHeaderTag`
- **BREAKING** some paths changed from `multiboot2_header::header` to `multiboot2_header::builder`
   -> thus, import paths are much more logically now
- internal code improvements

## v0.1.1 (2022-05-02)
- fixed a bug that prevented the usage of the crate in `no_std` environments
- added a new default `builder`-feature to Cargo which requires the `alloc`-crate
  (this feature can be disabled which will also remove the dependency to the `alloc` crate)

## v0.1.0 (2021-10-08)
- initial release

## v0.0.0
Empty release to save the name on crates.io
