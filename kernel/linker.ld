ENTRY(_start)

SECTIONS
{
    . = 0x10000;

    .text :
    {
        *(.text._start)
        *(.text .text.*)
        *(.rodata .rodata.*)
        *(.data .data.*)
    }

    .bss :
    {
        sbss = .;
        *(.bss .bss.*)
        ebss = .;
    }

    /DISCARD/ :
    {
        *(.eh_frame*)
        *(.comment)
        *(.note*)
    }
}
