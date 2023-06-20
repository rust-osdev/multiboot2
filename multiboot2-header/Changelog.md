# CHANGELOG for crate `multiboot2-header`

## 0.3.0 (xxxx-xx-xx)
- **BREAKING** MSRV is 1.68.0
- **BREAKING** renamed the `std` feature to `alloc`
- **BREAKING** bumped dependency to `multiboot2@v0.16.0`
- **BREAKING** renamed `MULTIBOOT2_HEADER_MAGIC` to `MAGIC`
- **BREAKING** renamed `Multiboot2HeaderBuilder` to `HeaderBuilder`
- **BREAKING** renamed `from_addr` to `load`. The function now consumes a ptr.
- added the optional `unstable` feature (requires nightly)
  - implement `core::error::Error` for `LoadError`

## 0.2.0 (2022-05-03)
- **BREAKING** renamed `EntryHeaderTag` to `EntryAddressHeaderTag`
- **BREAKING** some paths changed from `multiboot2_header::header` to `multiboot2_header::builder`
   -> thus, import paths are much more logically now
- internal code improvements

## 0.1.1 (2022-05-02)
- fixed a bug that prevented the usage of the crate in `no_std` environments
- added a new default `builder`-feature to Cargo which requires the `alloc`-crate
  (this feature can be disabled which will also remove the dependency to the `alloc` crate)

## 0.1.0 (2021-10-08)
- initial release

## 0.0.0
Empty release to save the name on crates.io
