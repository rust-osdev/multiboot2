# Changelog for Crate `multiboot2-common`

## Unreleased

## v0.7.0 (2026-09-04)

- `new_boxed` now panics if the total structure size does not round-trip through
  `Header::set_size`/`Header::total_size`, for example due to a lossy
  `set_size` implementation. Previously, such a mismatch led to a `Box` whose
  layout disagrees with the allocation, which is undefined behavior on
  deallocation.

## v0.6.0 (2026-09-02)

- **Breaking:** Fixed undefined behavior when serializing stack-constructed
  structures with implicit trailing padding: `MaybeDynSized::as_bytes` now
  returns a plain `&[u8]` that covers exactly the structure size reported in the
  header (clamped to the allocation) instead of a `BytesRef` over the whole
  allocation, whose trailing padding is uninitialized memory for
  stack-constructed values. `payload()` follows suit. As a side effect,
  `clone_dyn` now preserves the reported size exactly instead of growing it to
  the padded allocation size.
- Added the `raw_type!` macro that generates an ABI-safe `#[repr(transparent)]`
  newtype plus a corresponding high-level open-set enum, including all
  conversions between them and the underlying integer.
- **Breaking:** `Header` and `MaybeDynSized` are now `unsafe` traits, as this
  crate creates references from raw memory based on their implementations. The
  safety contracts implementors must uphold are now documented.

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
