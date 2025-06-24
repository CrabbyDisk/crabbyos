ENTRY( _entry )

MEMORY {
  RAM (rwx) : ORIGIN = 0x80000000, LENGTH = 128M
}

SECTIONS {
  .text : ALIGN(4K) {
    PROVIDE(_text_start = .);
    *(.text.*)
    PROVIDE(_text_end = .);
  } > RAM

  .rodata : ALIGN(4K) {
    PROVIDE(_rodata_start = .);
    *(.rodata.*)
    PROVIDE(_rodata_end = .);
  } > RAM

  .data : ALIGN(4K) {
    PROVIDE(_data_start = .);
    *(.data)
    PROVIDE(_data_end = .);
  } > RAM

  .bss : ALIGN(4K) {
    PROVIDE(_bss_start = .);
    *(.bss)
    PROVIDE(_bss_end = .);
  } > RAM

  .stack (NOLOAD) : ALIGN(4K) {
    PROVIDE(_stack_start = .);
    . = . + 256K;
    PROVIDE(_stack_end = .);
  } > RAM

  .heap (NOLOAD) : ALIGN(4K) {
    PROVIDE(_heap_start = .);
    . = . + 64M;
    PROVIDE(_heap_end = .);
  } > RAM

  PROVIDE(_uart = 0x10000000);
}
