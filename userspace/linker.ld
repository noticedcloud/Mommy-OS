ENTRY(_start)

SECTIONS {
    . = 0;
    .text : {
        *(.text.entry)
        *(.text*)
    }
    .rodata : {
        *(.rodata*)
    }
    .data : {
        *(.data*)
    }
    .bss : {
        *(.bss*)
    }
    /DISCARD/ : {
        *(.eh_frame*)
        *(.comment*)
        *(.note*)
        *(.dyn*)
        *(.gnu*)
        *(.hash*)
    }
}
