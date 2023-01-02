MEMORY
{
  flash : ORIGIN = 0x00000000 + SIZEOF_HEADERS, LENGTH = 32k
}

ENTRY(_start)

SECTIONS
{
  .text :
  {
    *(.rodata .rodata.*)
    *(.text .text.*)
  } >flash
}
