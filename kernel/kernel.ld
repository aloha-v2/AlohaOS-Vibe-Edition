/* Link the kernel at a fixed physical address. UEFI identity-maps RAM, so
 physical == virtual here and the loader can place segments verbatim. */
ENTRY(_start)

SECTIONS {
 . = 0x200000;

 __text_start = .;
 .text ALIGN(4K) : {
 *(.text .text.*)
 }
 __text_end = .;

 .rodata ALIGN(4K) : {
 *(.rodata .rodata.*)
 }

 .data ALIGN(4K) : {
 *(.data .data.*)
 }

 .bss ALIGN(4K) : {
 *(.bss .bss.*)
 *(COMMON)
 }

 /DISCARD/ : {
 *(.eh_frame)
 *(.eh_frame_hdr)
 *(.note .note.*)
 *(.comment)
 }
}
