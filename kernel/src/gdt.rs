use core::arch::asm;
#[repr(C, packed)]
struct GdtPointer {
    limit: u16,
    base: u64,
}
static mut GDT: [u64; 4] = [
    0x0000000000000000,
    0x00209a0000000000,
    0x0000920000000000,
    0x0000000000000000,
];
pub fn init_gdt() {
    unsafe {
        let gdt_ptr = GdtPointer {
            limit: (core::mem::size_of::<[u64; 4]>() - 1) as u16,
            base: &raw const GDT as u64,
        };
        asm!(
            "lgdt [{}]",
            "push 0x08",
            "lea rax, [rip + 2f]",
            "push rax",
            "retfq",
            "2:",
            "mov ax, 0x10",
            "mov ds, ax",
            "mov es, ax",
            "mov ss, ax",
            "mov fs, ax",
            "mov gs, ax",
            in(reg) &gdt_ptr,
            options(readonly, preserves_flags)
        );
    }
}
