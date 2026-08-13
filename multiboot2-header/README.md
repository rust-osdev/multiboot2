# multiboot2-header

[![crates.io](https://img.shields.io/crates/v/multiboot2-header.svg)](https://crates.io/crates/multiboot2-header)
[![docs](https://docs.rs/multiboot2-header/badge.svg)](https://docs.rs/multiboot2-header/)

Convenient and safe parsing of Multiboot2 Header structures and the
contained header tags. Usable in `no_std` environments, such as a
bootloader. The default `builder` feature also allows the construction of
the corresponding structures.

## Design

For every Multiboot2 header structure, there is an ABI-compatible rusty type.
This enables a zero-copying parsing design while also enabling the creation of
these structures via convenient constructors for the corresponding types.

## Use-Cases

What this library is good for:

- construct a Multiboot2 header at runtime (constructing one at build time with
  macros is not done yet, contributions are welcome!)
- write a Multiboot2 bootloader that parses a Multiboot2 header
- understand Multiboot2 headers better
- analyze Multiboot2 headers at runtime

## Features and `no_std` Compatibility

This library is always `no_std`. The default `builder` feature enables `alloc`;
using it requires an `#[global_allocator]`. Remove that feature if you do not
need to construct headers.

```toml
# Without the `builder` feature or the `alloc` crate:
multiboot2-header = { version = "<latest>", default-features = false }
# With the default `builder` feature (requires the `alloc` crate):
multiboot2-header = "<latest>"
```

## Example 1: Builder + Parse

```rust
use multiboot2_header::{
    Builder, HeaderTagFlag, HeaderTagISA, InformationRequestHeaderTag,
    MaybeDynSized, MbiTagType, Multiboot2Header, RelocatableHeaderTag,
    RelocatableHeaderTagPreference,
};

/// Small example that creates a Multiboot2 header and parses it afterwards.
fn main() {
    let header_bytes = Builder::new(HeaderTagISA::I386)
        .relocatable_tag(RelocatableHeaderTag::new(
            HeaderTagFlag::Required,
            0x1337,
            0xdeadbeef,
            4096,
            RelocatableHeaderTagPreference::None,
        ))
        .information_request_tag(InformationRequestHeaderTag::new(
            HeaderTagFlag::Required,
            &[
                MbiTagType::Cmdline.into(),
                MbiTagType::BootLoaderName.into(),
            ],
        ))
        .build();

    let header = unsafe { Multiboot2Header::load(header_bytes.as_ptr()) }.unwrap();
    println!("{header:#?}");
}
```

## Example 2: Multiboot2 header as static data in Rust file

You can use the builder, construct a Multiboot2 header, write it to a file and
include it like this:

```rust
#[used]
#[unsafe(no_mangle)]
#[link_section = ".text.multiboot2_header"]
static MULTIBOOT2_HDR: [u8; 64] = *include_bytes!("mb2_hdr_dump.bin");
```

You may need a special linker script to place this symbol in the first 32768
bytes of the ELF. See Multiboot2 specification.

## MSRV

The MSRV is 1.85.1 stable.

## License & Contribution

See main [README](https://github.com/rust-osdev/multiboot2/blob/main/README.md)
file.
