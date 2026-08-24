# Changelog for Crate `multiboot2-common`

## Unreleased

## v0.5.0 (2026-08-24)

- Fixed undefined behavior in `DynSizedStructure::cast`: the target size is now
  validated before the reference is created, so casting a too-small (e.g.
  malformed or truncated) structure panics instead of retagging out of bounds.
- Fixed undefined behavior in `new_boxed`: the trailing alignment padding is now
  zero-initialized, so reading it back through `MaybeDynSized::as_bytes` or
  `payload` no longer reads uninitialized memory.
- Fixed undefined behavior in `DynSizedStructure::ref_from_ptr`: a misaligned
  pointer is now reported as `MemoryError::WrongAlignment` before the header is
  dereferenced, instead of forming a misaligned reference. This makes the
  documented "misaligned pointer returns an error" contract of
  `BootInformation::load` and `Header::load` hold.

## v0.4.1 (2026-08-13)

- Clarified feature, stability, and memory-safety documentation.

## v0.4.0 (2026-06-23)

- **Breaking:** `Header` now requires `total_size()` and derives
  `payload_len()` from it.
- Added validation for complete padded tag sequences.
- Added size details to memory validation errors.
- Fixed validation for dynamically sized structures whose reported total size
  exceeds the available buffer.
- Small code improvements

## v0.3.0 (2025-06-01)

- **Breaking:** Removed the optional `unstable` feature (required nightly)
  - `core::error::Error` is now implemented unconditionally
- **Breaking:** The MSRV is now 1.85

## v0.2.1 (2024-09-19)

- Documentation improvements

## v0.2.0 (2024-09-17)

- dependency updates
- **Breaking:** MSRV is now 1.75
- misc metadata fixes

## v0.1.2 (2024-08-24)

- Documentation improvements

## 0.1.0 / 0.1.1 (2024-08-20)

Initial release.
