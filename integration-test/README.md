# Integrationtests

This directory contains integration tests for the `multiboot2` and the
`multiboot2-header` crate. The integration tests start a QEMU VM and do certain
checks at runtime. If something fails, they instruct QEMU to exit with an error
code. All output of the VM is printed to the screen.

The `bins` directory contains Rust binaries that **are** the tests. The `tests`
directory contains test definitions, run scripts, and other relevant files. The
main entry to run all tests is `./run.sh` in this directory.

## TL;DR

- `$ nix-shell --run ./run.sh` to execute the integration tests with Nix
  (recommended)
    - `$ nix-shell --run "integration-test/run.sh"` to run the test from the
      project root
- `$ ./run.sh` to execute the integration tests (you have to get the
  dependencies manually)

## Prerequisites

The tests are executed best when using [`nix`](https://nixos.org/)/`nix-shell`
to get the relevant tools. Otherwise, please make sure the following packages
are available:

- grub helper tools (grub-file)
- rustup
- OVMF
- QEMU
- xorriso
