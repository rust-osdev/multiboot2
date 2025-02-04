#!/usr/bin/env bash

# http://redsymbol.net/articles/unofficial-bash-strict-mode/
set -euo pipefail
IFS=$'\n\t'

DIR=$(dirname "$(realpath "$0")")
cd "$DIR" || exit

BINS_DIR=bins/target/x86-unknown-none/release
ANSI_HIGHLIGHT="\e[1;32m" # green + bold
ANSI_HIGHLIGHT_ERROR="\e[1;31m" # red + bold
ANSI_RESET="\e[0m"
QEMU_ARGS_BASE=(
   -machine q35,accel=kvm
   -m 128m # for OVMF, we need more than just 24
   -debugcon stdio
   -serial file:serial.txt
   -no-reboot
   -device isa-debug-exit,iobase=0xf4,iosize=0x04
   -display none `# relevant for the CI`
)

function fn_main() {
    git submodule update --init
    fn_build_limine_hosttool
    fn_build_rust_bins

    fn_test_payload
    fn_test_loader
}

function fn_build_limine_hosttool() {
    cd limine-bootloader
    make
    test -f ./limine
    file --brief ./limine | grep -q "ELF 64-bit LSB executable"
    cd -
}

function fn_build_rust_bins() {
    cd "bins"
    cargo --version
    cargo build --release --verbose
    cd -

    echo "Verifying multiboot2_chainloader ..."
    test -f $BINS_DIR/multiboot2_chainloader
    file --brief $BINS_DIR/multiboot2_chainloader | grep -q "ELF 32-bit LSB executable"
    grub-file --is-x86-multiboot2 $BINS_DIR/multiboot2_chainloader

    echo "Verifying multiboot2_payload ..."
    test -f $BINS_DIR/multiboot2_payload
    file --brief $BINS_DIR/multiboot2_payload | grep -q "ELF 32-bit LSB executable"
    grub-file --is-x86-multiboot2 $BINS_DIR/multiboot2_payload
}

function fn_prepare_test_vol() {
    TEST_VOL="$TEST_DIR/.vol"
    rm -rf $TEST_VOL
    mkdir -p $TEST_VOL
    cp $TEST_DIR/limine.cfg $TEST_VOL

    # copy limine artifacts
    mkdir -p $TEST_VOL/limine
    cp limine-bootloader/limine-bios-cd.bin $TEST_VOL/limine
    cp limine-bootloader/limine-bios.sys $TEST_VOL/limine
    cp limine-bootloader/limine-uefi-cd.bin $TEST_VOL/limine

    mkdir -p $TEST_VOL/EFI/BOOT
    cp limine-bootloader/BOOTX64.EFI $TEST_VOL/EFI_BOOT
}



# Builds a hybrid-bootable image using Limine as bootloader. Expects that
# all relevant files are in the directory describing the root volume.
function fn_build_limine_iso() {
    xorriso -as mkisofs -b limine/limine-bios-cd.bin \
            -no-emul-boot -boot-load-size 4 -boot-info-table \
            --efi-boot limine/limine-uefi-cd.bin \
            -efi-boot-part --efi-boot-image --protective-msdos-label \
            $TEST_VOL -o $TEST_DIR/image.iso 2>/dev/null

    ./limine-bootloader/limine bios-install $TEST_DIR/image.iso 2>/dev/null
}

function fn_run_qemu() {
    set +e

    # As QEMU can't print serial and debugcon to stdout simultaneously, I
    # add a background task watching serial.txt
    rm serial.txt
    touch serial.txt
    tail -f serial.txt &

    qemu-system-x86_64 "${QEMU_ARGS[@]}"
    EXIT_CODE=$?
    # Custom exit code used by the integration test to report success.
    QEMU_EXIT_SUCCESS=73

    set -e

    echo "#######################################"
    if [[ $EXIT_CODE -eq $QEMU_EXIT_SUCCESS ]]; then
        echo -e "${ANSI_HIGHLIGHT}SUCCESS${ANSI_RESET}"
        echo # newline
    else
        echo -e "${ANSI_HIGHLIGHT_ERROR}FAILED - Integration Test 'multiboot2-header'${ANSI_RESET}"
        exit "$EXIT_CODE"
    fi
}

function fn_run_test_bios() {
    local ISO=$1
    local QEMU_ARGS=("${QEMU_ARGS_BASE[@]}") # copy array
    local QEMU_ARGS+=(
        -cdrom "$ISO"
    )
    echo -e "Running '${ANSI_HIGHLIGHT}$ISO${ANSI_RESET}' in QEMU (with legacy BIOS firmware)"
    fn_run_qemu
}

function fn_run_test_uefi() {
    local ISO=$1
    local QEMU_ARGS=("${QEMU_ARGS_BASE[@]}") # copy array
    local QEMU_ARGS+=(
        # Usually, this comes from the Nix shell.
        -bios $OVMF
        -cdrom "$ISO"
    )
    echo -e "Running '${ANSI_HIGHLIGHT}$ISO${ANSI_RESET}' in QEMU (with UEFI/OVMF firmware)"
    fn_run_qemu $QEMU_ARGS
}

function fn_test_payload() {
    local TEST_DIR=tests/01-boot-payload
    fn_prepare_test_vol

    cp $BINS_DIR/multiboot2_payload $TEST_VOL/kernel

    fn_build_limine_iso

    fn_run_test_bios $TEST_DIR/image.iso
    fn_run_test_uefi $TEST_DIR/image.iso
}

# Tests the loader by chainloading the Multiboot2 payload.
function fn_test_loader() {
    local TEST_DIR=tests/02-boot-loader-and-chainload
    fn_prepare_test_vol

    cp $BINS_DIR/multiboot2_chainloader $TEST_VOL/kernel
    cp $BINS_DIR/multiboot2_payload $TEST_VOL/payload

    fn_build_limine_iso

    fn_run_test_bios $TEST_DIR/image.iso
    fn_run_test_uefi $TEST_DIR/image.iso
}

fn_main
