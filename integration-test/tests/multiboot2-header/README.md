# multiboot2-header - Integration Test

This integration test loads the `multiboot2_chainloader` binary as Multiboot1
payload using QEMU. The `multiboot2_payload` binary is passed as boot module.
The `multiboot2_chainloader` behaves as bootloader and eventually loads
`multiboot2_payload` into the memory. `multiboot2_payload` figures out during
runtime whether it was loaded by GRUB or by the chainloader.
