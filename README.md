# Multiboot2: MBI + Header

This repository contains the crates `multiboot2` and `multiboot2-header`.
Please check their individual README-files ([multiboot2](multiboot2/README.md),
[multiboot2-header](multiboot2-header/README.md)).

The `multiboot2` crate helps to parse the Multiboot2 information structure
(MBI) and is relevant in kernels, that get booted by a bootloader such as
GRUB, for example. `multiboot2-header` is relevant, if you want to write a bootloader that provides a MBI to a payload for
example.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
