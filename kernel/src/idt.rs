use crate::serial::outb;
use core::arch::asm;
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct IdtEntry {
    pub offset_low: u16,
    pub selector: u16,
    pub ist: u8,
    pub flags: u8,
    pub offset_mid: u16,
    pub offset_high: u32,
    pub reserved: u32,
}
#[repr(C, packed)]
struct IdtPointer {
    limit: u16,
    base: u64,
}
pub static mut IDT: [IdtEntry; 256] = [IdtEntry {
    offset_low: 0,
    selector: 0,
    ist: 0,
    flags: 0,
    offset_mid: 0,
    offset_high: 0,
    reserved: 0,
}; 256];
static mut IDT_PTR: IdtPointer = IdtPointer { limit: 0, base: 0 };
#[inline(never)]
pub fn init_idt() {
    unsafe {
        crate::serial::print_serial(b"Setting up IDT...\n");
        IDT_PTR.limit = (core::mem::size_of::<[IdtEntry; 256]>() - 1) as u16;
        IDT_PTR.base = &raw const IDT as u64;
        crate::serial::print_serial(b"IDT Base: ");
        crate::serial::print_hex(IDT_PTR.base);
        crate::serial::print_serial(b"\n");
        asm!("lidt [{}]", in(reg) &raw const IDT_PTR, options(readonly, preserves_flags));
        crate::serial::print_serial(b"LIDT Executed.\n");
    }
}
pub unsafe fn set_idt_gate(n: usize, handler: u64) {
    IDT[n].offset_low = (handler & 0xFFFF) as u16;
    IDT[n].selector = 0x08;
    IDT[n].ist = 0;
    IDT[n].flags = 0x8E;
    IDT[n].offset_mid = ((handler >> 16) & 0xFFFF) as u16;
    IDT[n].offset_high = (handler >> 32) as u32;
    IDT[n].reserved = 0;
}
pub fn pic_remap() {
    outb(0x20, 0x11);
    outb(0xA0, 0x11);
    outb(0x21, 0x20);
    outb(0xA1, 0x28);
    outb(0x21, 0x04);
    outb(0xA1, 0x02);
    outb(0x21, 0x01);
    outb(0xA1, 0x01);
    outb(0x21, 0xFC);
    outb(0xA1, 0xFF);
}
