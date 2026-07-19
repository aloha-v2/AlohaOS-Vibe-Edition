ENTRY(_start)
SECTIONS { . = 0x0000008000000000; .text : ALIGN(16) { *(.text._start) *(.text*) } .rodata : ALIGN(16) { *(.rodata*) } }
