ENTRY(_start)

PHDRS
{
  user_text PT_LOAD FLAGS(5);
}

SECTIONS
{
  . = 0x0000008000000000;
  .text : ALIGN(16) { *(.text._start) *(.text*) } :user_text
  .rodata : ALIGN(16) { *(.rodata*) } :user_text
  /DISCARD/ : { *(.eh_frame*) *(.comment*) }
}
