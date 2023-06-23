#!/usr/bin/env bash

# http://redsymbol.net/articles/unofficial-bash-strict-mode/
set -euo pipefail
IFS=$'\n\t'

DIR=$(dirname "$(realpath "$0")")
cd "$DIR" || exit

function fn_main() {
    fn_build_rust_bins
    fn_multiboot2_integrationtest
    fn_multiboot2_header_integrationtest
}

function fn_build_rust_bins() {
    cd "bins"
    cargo build --release
    cd "$DIR"
}

function fn_multiboot2_integrationtest() {
    cd tests/multiboot2
    ./build_img.sh
    ./run_qemu.sh
    cd "$DIR"
}

function fn_multiboot2_header_integrationtest() {
    cd tests/multiboot2-header
    ./run_qemu.sh
    cd "$DIR"
}

fn_main
