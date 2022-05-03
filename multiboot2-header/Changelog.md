# CHANGELOG for crate `multiboot2-header`

## v0.2.0 (2022-05-03)
- **breaking** renamed `EntryHeaderTag` to `EntryAddressHeaderTag`

## v0.1.1 (2022-05-02)
- fixed a bug that prevented the usage of the crate in `no_std` environments
- added a new default `builder`-feature to Cargo which requires the `alloc`-crate
  (this feature can be disabled which will also remove the dependency to the `alloc` crate)

## v0.1.0 (2021-10-08)
- initial release

## v0.0.0
Empty release to save to name on crates.io
