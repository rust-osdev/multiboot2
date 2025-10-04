#!/usr/bin/env bash

# http://redsymbol.net/articles/unofficial-bash-strict-mode/
set -euo pipefail
IFS=$'\n\t'

DIR=$(dirname "$(realpath "$0")")
cd "$DIR" || exit

# rustc 1.89 (nightly).
RUSTUP_NIGHTLY_TOOLCHAIN="nightly-2025-05-31"

function fn_main() {
    fn_prepare_rustup
    fn_build_rust_bins
    fn_multiboot2_integrationtest
    fn_multiboot2_header_integrationtest
}

# We prefer this over a rustup-toolchain.toml as we need the nightly-toolchain
# only for the integration tests but not the actual crate.
function fn_prepare_rustup() {
    rustup toolchain add "$RUSTUP_NIGHTLY_TOOLCHAIN"
    rustup component add rust-src --toolchain "$RUSTUP_NIGHTLY_TOOLCHAIN"
}

function fn_build_rust_bins() {
    RUSTUP_TOOLCHAIN="$RUSTUP_NIGHTLY_TOOLCHAIN" \
    cargo build \
        --verbose \
        --target ./bins/x86-unknown-none.json \
        -Z build-std=core,alloc,compiler_builtins \
        -Z build-std-features=compiler-builtins-mem \
        --profile release-integration-test \
        -p multiboot2_chainloader \
        -p multiboot2_payload
}

function fn_multiboot2_integrationtest() {
    cd tests/multiboot2
    ./build_img.sh
    ./run_qemu.sh
    cd -
}

function fn_multiboot2_header_integrationtest() {
    cd tests/multiboot2-header
    ./run_qemu.sh
    cd -
}

fn_main
