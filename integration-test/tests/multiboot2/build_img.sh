#!/usr/bin/env bash

# This script builds a bootable image. It bundles the test binary into a GRUB
# installation. The GRUB installation is configured to chainload the binary
# via Multiboot2.

# http://redsymbol.net/articles/unofficial-bash-strict-mode/
set -euo pipefail
IFS=$'\n\t'

DIR=$(dirname "$(realpath "$0")")
cd "$DIR" || exit

TARGET_DIR="../../../target/x86-unknown-none/release-integration-test"
MULTIBOOT2_PAYLOAD_PATH="$TARGET_DIR/multiboot2_payload"

echo "Verifying that the binary is a multiboot2 binary..."
grub-file --is-x86-multiboot2 "$MULTIBOOT2_PAYLOAD_PATH"

# Delete previous state.
rm -rf .vol

mkdir -p .vol/boot/grub
cp grub.cfg .vol/boot/grub
cp "$MULTIBOOT2_PAYLOAD_PATH" .vol

# Create a GRUB image with the files in ".vol" being embedded.
echo "Creating bootable image..."
grub-mkrescue -o "grub_boot.img" ".vol"
