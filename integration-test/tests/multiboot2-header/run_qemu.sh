#!/usr/bin/env bash

# This script starts a bootable image in QEMU using legacy BIOS boot.

# http://redsymbol.net/articles/unofficial-bash-strict-mode/
set -euo pipefail
IFS=$'\n\t'

DIR=$(dirname "$(realpath "$0")")
cd "$DIR" || exit

TARGET_DIR="../../../target/x86-unknown-none/release-integration-test"
CHAINLOADER="$TARGET_DIR/multiboot2_chainloader"
PAYLOAD="$TARGET_DIR/multiboot2_payload"
# add "-d int \" to debug CPU exceptions
# "-display none" is necessary for the CI but locally the display and the
#   combat monitor are really helpful

set +e
qemu-system-x86_64 \
    -kernel "$CHAINLOADER" \
    -append "chainloader" \
    -initrd "$PAYLOAD multiboot2 payload" \
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
    echo "SUCCESS - Integration Test 'multiboot2-header'"
    exit 0
else
    echo "FAILED - Integration Test 'multiboot2-header'"
    exit "$EXIT_CODE"
fi
