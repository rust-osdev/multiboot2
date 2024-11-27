# The assembly code uses the GNU Assembly (GAS) flavor with Intel noprefix
# syntax.

# Symbol from main.rs
.EXTERN start

.code32
.align 8
.section .multiboot2_header

    mb2_header_start:
        .long  0xe85250d6                # magic number
        .long  0                         # architecture 0 (protected mode i386)
        .long  mb2_header_end - mb2_header_start # header length
        # checksum
        .long  0x100000000 - (0xe85250d6 + 0 + (mb2_header_end - mb2_header_start))

        # REQUIRED END TAG
        .align 8
        .Lmb2_header_tag_end_start:
            .word  0       # type  (16bit)
            .word  0       # flags (16bit)
            .long  .Lmb2_header_tag_end_end - .Lmb2_header_tag_end_start # size  (32bit)
        .Lmb2_header_tag_end_end:
    mb2_header_end:
