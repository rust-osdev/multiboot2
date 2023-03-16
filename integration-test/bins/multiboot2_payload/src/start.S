# Symbol from main.rs
.extern rust_entry

.code32
.align 8
.section .text

.global start
start:
    # Prepare Multiboot2-handoff parameters for Rust
    mov     %eax,       %edi
    mov     %ebx,       %esi

    # Prepare stack + align it to 16 byte (for SSE registers)
    mov     $stack_end, %eax
    sub     $16,        %eax
    # x86 quirk: stack is n-aligned at address x when %esp+$8 is n-aligned
    add     $8,         %eax

    # Set stack
    mov     %eax,       %esp
    mov     %eax,       %ebp

    # Enable SSE.
    # Strictly speaking, this is not necessary, but I activated SSE in the
    # compiler spec json file. Rustc/LLVM produces SSE coe for example from the
    # core::fmt code.
    mov %cr0,       %eax
    and $0xFFFB,    %ax     # clear coprocessor emulation CR0.EM
    or  $0x2,       %ax		# set coprocessor monitoring  CR0.MP
    mov %eax,       %cr0
    mov %cr4,       %eax
    or  $(3 << 9),  %ax		# set CR4.OSFXSR and CR4.OSXMMEXCPT
    mov %eax,       %cr4

    # x86 (i386) calling convention:
    # 1. prepare stackframe pointer
    # 2. push arguments on stack in reverse order
    push %ebp
    mov %esp    , %ebp
    push %esi
    push %edi
    call rust_entry
    ud2

.section .data

# 16K natural-aligned stack.
.align 16384
stack_begin:
    .zero 16384
stack_end:
