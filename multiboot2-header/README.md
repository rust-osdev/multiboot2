# multiboot2-header

[![crates.io](https://img.shields.io/crates/v/multiboot2-header.svg)](https://crates.io/crates/multiboot2-header)
[![docs](https://docs.rs/multiboot2-header/badge.svg)](https://docs.rs/multiboot2-header/)

Convenient and safe parsing of Multiboot2 Header structures and the
contained header tags. Usable in `no_std` environments, such as a
bootloader. An optional `builder` feature also allows the construction of
the corresponding structures.

## Design

For every Multiboot2 header structure, there is an ABI-compatible rusty type.
This enables a zero-copying parsing design while also enabling the creation of
these structures via convenient constructors for the corresponding types.

## Use-Cases

What this library is good for:

- construct a Multiboot2 header at runtime (constructing one at build-time with
  macros is not done yet, contributions are welcome!)
- write a Multiboot2-bootloader that parses a Multiboot2-header
- understanding Multiboot2 headers better
- analyze Multiboot2 headers at runtime

## Features and `no_std` Compatibility

This library is always `no_std` without `alloc`. However, the default `builder`-
feature requires the `alloc`-crate and an `#[global_allocator]` to be available.
You need the `builder` only if you want to construct new headers at runtime.
For parsing, the feature is not relevant, and you can deactivate it.

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

You can use the builder, construct a Multiboot2 header, write it to a file and
include it like this:

```
#[used]
#[no_mangle]
#[link_section = ".text.multiboot2_header"]
static MULTIBOOT2_HDR: [u8; 64] = *include_bytes!("mb2_hdr_dump.bin");
```

You may need a special linker script to place this symbol in the first 32768
bytes of the ELF. See Multiboot2 specification.

## MSRV

The MSRV is 1.70.0 stable.

## License & Contribution

See main [README](https://github.com/rust-osdev/multiboot2/blob/main/README.md)
file.
