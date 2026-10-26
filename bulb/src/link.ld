OUTPUT_FORMAT(elf64-x86-64)
ENTRY(_start)

PHDRS
{
    text PT_LOAD FLAGS(5);
    rodata PT_LOAD FLAGS(4);
    data PT_LOAD FLAGS(6);
}

SECTIONS
{
    . = 0xFFFFFFFF80000000;

    .limine_requests : ALIGN(8) {
        KEEP(*(.limine_requests))
    } :text

    .text : ALIGN(4096) {
        *(.text .text.*)
    } :text

    . = ALIGN(4096);

    .rodata : ALIGN(4096) {
        *(.rodata .rodata.*)
    } :rodata

    . = ALIGN(4096);

    .data : ALIGN(4096) {
        *(.data .data.*)
    } :data

    .bss : ALIGN(4096) {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        . = ALIGN(8);
        __bss_end = .;
    } :data

    /DISCARD/ : {
        *(.eh_frame .eh_frame_hdr .note .note.* .comment)
    }
}
