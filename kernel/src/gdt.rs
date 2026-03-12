use core::arch::asm;
#[repr(C, packed)]

pub struct Tss {
    reserved1: u32,
    pub rsp0: u64,
    pub rsp1: u64,
    pub rsp2: u64,
    reserved2: u64,
    pub ist: [u64; 7],
    reserved3: u64,
    reserved4: u16,
    pub iomap_base: u16,
}

#[repr(C, packed)]

struct GdtPointer {
    limit: u16,
    base: u64,
}

pub static mut TSS: Tss = Tss {
    reserved1: 0,
    rsp0: 0,
    rsp1: 0,
    rsp2: 0,
    reserved2: 0,
    ist: [0; 7],
    reserved3: 0,
    reserved4: 0,
    iomap_base: 104,
};

static mut GDT: [u64; 9] = [
    0x0000000000000000,
    0x0020980000000000,
    0x0000920000000000,
    0x0000000000000000,
    0x0000f20000000000,
    0x0020f80000000000,
    0x0000000000000000,
    0x0000000000000000,
    0x0000000000000000,
];

pub fn init_gdt() {
    unsafe {
        let tss_addr = &raw const TSS as u64;
        let tss_desc_low = (103 & 0xFFFF)
            | ((tss_addr & 0xFFFF) << 16)
            | (((tss_addr >> 16) & 0xFF) << 32)
            | (0x89 << 40)
            | ((0 & 0xF) << 48)
            | (((tss_addr >> 24) & 0xFF) << 56);
        let tss_desc_high = tss_addr >> 32;

        GDT[7] = tss_desc_low;
        GDT[8] = tss_desc_high;

        let gdt_ptr = GdtPointer {
            limit: (core::mem::size_of::<[u64; 9]>() - 1) as u16,
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
            "mov fs, ax",
            "mov gs, ax",
            "mov ss, ax",
            "mov ax, 0x38",
            "ltr ax",
            in(reg) &gdt_ptr,
            options(readonly, preserves_flags)
        );
    }
}
