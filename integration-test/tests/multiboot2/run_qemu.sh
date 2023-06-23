#!/usr/bin/env bash

# This script starts a bootable image in QEMU using legacy BIOS boot.

# http://redsymbol.net/articles/unofficial-bash-strict-mode/
set -euo pipefail
IFS=$'\n\t'

DIR=$(dirname "$(realpath "$0")")
cd "$DIR" || exit

BOOT_IMAGE="grub_boot.img"

# add "-d int \" to debug CPU exceptions
# "-display none" is necessary for the CI but locally the display and the
#   combat monitor are really helpful

set +e
qemu-system-x86_64 \
    -boot d \
    -cdrom "$BOOT_IMAGE" \
    -m 24m \
    -debugcon stdio \
    -no-reboot \
    -device isa-debug-exit,iobase=0xf4,iosize=0x04 \
    -display none `# relevant for the CI`

EXIT_CODE=$?
# Custom exit code used by the integration test to report success.
QEMU_EXIT_SUCCESS=73


echo "#######################################"
if [[ $EXIT_CODE -eq $QEMU_EXIT_SUCCESS ]]; then
    echo "SUCCESS - Integration Test 'multiboot2'"
    exit 0
else
    echo "FAILED - Integration Test 'multiboot2'"
    exit "$EXIT_CODE"
fi
