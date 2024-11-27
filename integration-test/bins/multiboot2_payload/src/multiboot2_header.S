# Multiboot2 Header definition.
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

        # OPTIONAL MULTIBOOT2 TAGS (additional to required END TAG)
        # ------------------------------------------------------------------------------------
        .align 8
        .Lmb2_header_tag_information_request_start:
            .word  1       # type  (16bit)
            .word  1       # flags (16bit)
            .long  .Lmb2_header_tag_information_request_end - .Lmb2_header_tag_information_request_start # size  (32bit)
            .long  1
            .long  2
            .long  3
            .long  4
            .long  5
            .long  6
            .long  7
            .long  8
            .long  9
            .long  10
            .long  11
            .long  12
            # .long  13 GRUB reports: not supported
            .long  14
            .long  15
            .long  16
            .long  17
            .long  18
            .long  19
            .long  20
            .long  21
            .long
        .Lmb2_header_tag_information_request_end:

        .align 8
        .Lmb2_header_tag_module_alignment_start:
            .word  7       # type  (16bit)
            .word  0       # flags (16bit)
            .long  .Lmb2_header_tag_module_alignment_end - .Lmb2_header_tag_module_alignment_start # size  (32bit)
            .long start
        .Lmb2_header_tag_module_alignment_end:
        # ------------------------------------------------------------------------------------

        # REQUIRED END TAG
        .align 8
        .Lmb2_header_tag_end_start:
            .word  0       # type  (16bit)
            .word  0       # flags (16bit)
            .long  .Lmb2_header_tag_end_end - .Lmb2_header_tag_end_start # size  (32bit)
        .Lmb2_header_tag_end_end:
    mb2_header_end:
