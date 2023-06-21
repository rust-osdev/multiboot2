# multiboot2-header
![Build](https://github.com/rust-osdev/multiboot2/actions/workflows/rust.yml/badge.svg)
[![crates.io](https://img.shields.io/crates/v/multiboot2-header.svg)](https://crates.io/crates/multiboot2-header)
[![docs](https://docs.rs/multiboot2-header/badge.svg)](https://docs.rs/multiboot2-header/)

Rust library with type definitions and parsing functions for Multiboot2 headers.
This library is `no_std` and can be used in bootloaders.

What this library is good for:
- writing a small binary which writes you a valid Multiboot2 header
  into a file (such as `header.bin`)
- understanding Multiboot2 headers better
- analyze Multiboot2 headers at runtime

What this library is not optimal for:
- compiling a Multiboot2 header statically into an object file using only Rust code

## Features and `no_std` Compatibility
This library is always `no_std`. However, the default `builder`-feature requires
the `alloc`-crate to be available. You need the `builder` only if you want to
construct new headers at run time. For parsing, this is not relevant, and you
can deactivate the default feature.

```toml
# without `builder`-feature (and without `alloc`-crate)
multiboot2-header = { version = "<latest>", default-features = false }
# else (requires `alloc`-crate)
multiboot2-header = "<latest>"
```

## Example 1: Builder + Parse
```rust
use multiboot2_header::builder::{InformationRequestHeaderTagBuilder, Multiboot2HeaderBuilder};
use multiboot2_header::{HeaderTagFlag, HeaderTagISA, MbiTagType, RelocatableHeaderTag, RelocatableHeaderTagPreference, Multiboot2Header};

/// Small example that creates a Multiboot2 header and parses it afterwards.
fn main() {
    // We create a Multiboot2 header during runtime here. A practical example is that your
    // program gets the header from a file and parses it afterwards.
    let mb2_hdr_bytes = Multiboot2HeaderBuilder::new(HeaderTagISA::I386)
        .relocatable_tag(RelocatableHeaderTag::new(
            HeaderTagFlag::Required,
            0x1337,
            0xdeadbeef,
            4096,
            RelocatableHeaderTagPreference::None,
        ))
        .information_request_tag(
            InformationRequestHeaderTagBuilder::new(HeaderTagFlag::Required)
                .add_irs(&[MbiTagType::Cmdline, MbiTagType::BootLoaderName]),
        )
        .build();

    // Cast bytes in vector to Multiboot2 information structure
    let mb2_hdr = unsafe { Multiboot2Header::from_addr(mb2_hdr_bytes.as_ptr().cast()) };
    println!("{:#?}", mb2_hdr);
}
```

## Example 2: Multiboot2 header as static data in Rust file
You can use the builder, construct a Multiboot2 header, write it to a file and include it like this:
```
#[used]
#[no_mangle]
#[link_section = ".text.multiboot2_header"]
static MULTIBOOT2_HDR: &[u8; 64] = include_bytes!("mb2_hdr_dump.bin");
```
You may need a special linker script to place this in a LOAD segment with a file offset with less than 32768 bytes.
See specification.

## MSRV
The MSRV is 1.68.0 stable.

## License & Contribution

See main [README](https://github.com/rust-osdev/multiboot2/blob/main/README.md) file.
