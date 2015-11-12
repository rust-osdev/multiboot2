# multiboot2-elf64
An experimental Multiboot 2 crate for ELF-64 kernels. It's very incomplete and completely untested. I wrote it for the [blog OS project](https://github.com/phil-opp/blog_os).

Contributions welcome! **If someone wants to maintain this crate, please contact me!**

It uses the Multiboot 1.6 specification at http://nongnu.askapache.com/grub/phcoder/multiboot.pdf and the ELF 64 specification at http://www.uclibc.org/docs/elf-64-gen.pdf.

Note that the multiboot specification for the ELF-sections tag seems to be wrong for ELF 64 kernels: The `num`, `entsize`, and `shndx` fields seem to be `u32` instead of `u16` (but I'm not sure on this).
