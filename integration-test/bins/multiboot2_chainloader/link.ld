ENTRY(start)

PHDRS
{
    /* PT_LOAD FLAGS (page table permissions) not necessary, as we perform
       legacy boot. Linker probably sets sensible defaults anyway. */
    kernel_rx     PT_LOAD;
    kernel_rw     PT_LOAD;
    kernel_ro     PT_LOAD;
}

SECTIONS {
    /* Chainloader linked at 8M, payload at 16M */
    .text 8M : AT(8M) ALIGN(4K)
    {
        KEEP(*(.multiboot_header));
        *(.text .text.*)
    } : kernel_rx

    .rodata :
    {
        *(.rodata .rodata.*)
    } : kernel_ro

    .data :
    {
        *(.data .data.*)
    } : kernel_rw

    .bss :
    {
        *(COMMON)
        *(.bss .bss.*)
    } : kernel_rw

    /DISCARD/ :
    {
        *(.eh_frame .eh_frame*)
    }
}
