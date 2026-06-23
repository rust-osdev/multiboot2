# Changelog for Crate `multiboot2-common`

## Unreleased

- **Breaking:** The MSRV is now 1.87.0
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
