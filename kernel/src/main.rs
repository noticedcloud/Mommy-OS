#![no_std]
#![no_main]
mod drivers;
pub mod elf;
pub mod font;
pub mod fs;
mod gdt;
mod idt;
mod interrupts;
mod keyboard;
pub mod net;
pub mod paging;
mod pmm;
mod power;
mod serial;
mod shell;
mod syscall;
mod task;
mod vesa;
mod vga;
use crate::gdt::init_gdt;
use crate::idt::init_idt;
use crate::serial::print_serial;
use crate::vga::print;
use core::arch::asm;
use core::fmt::Write;
use core::panic::PanicInfo;
#[no_mangle]
#[link_section = ".text._start"]
pub extern "C" fn _start() -> ! {
    unsafe {
        core::arch::asm!("cli");
    }
    extern "C" {
        static mut sbss: u8;
        static mut ebss: u8;
    }
    unsafe {
        let mut ptr = &raw mut sbss as *mut u8;
        let end = &raw mut ebss as *mut u8;
        while ptr < end {
            *ptr = 0;
            ptr = ptr.add(1);
        }
    }
    #[repr(align(16))]
    struct Stack(#[allow(dead_code)] [u8; 16384]);
    static mut STACK: Stack = Stack([0; 16384]);
    unsafe {
        let stack_top = &raw const STACK as *const _ as u64 + 16384;
        asm!("mov rsp, {}", in(reg) stack_top);
    }
    kernel_init();
    loop {
        unsafe { asm!("hlt") }
    }
}
fn kernel_init() {
    crate::vga::init();
    print_serial(b"\n--- KERNEL BOOT: START ---\n");
    init_descriptors();
    init_interrupts();
    init_memory();
    init_tasks();
    init_drivers();
    init_fs();
    init_shell();
    print(b"\nMommy is listening, write darling <3 ");
    crate::vga::set_input_start();
    enable_interrupts();
}
fn init_descriptors() {
    print_serial(b"[INIT] Descriptors (GDT/IDT)... ");
    crate::gdt::init_gdt();
    crate::idt::init_idt();
    print_serial(b"OK\n");
}
fn init_interrupts() {
    print_serial(b"[INIT] Interrupts (PIC/Syscalls/PIT)... ");
    crate::idt::pic_remap();
    unsafe {
        crate::idt::set_idt_gate(0x20, crate::interrupts::timer_handler_stub as u64);
        crate::idt::set_idt_gate(0x21, crate::interrupts::keyboard_handler_stub as u64);
        crate::idt::set_idt_gate(0x03, crate::interrupts::generic_handler_stub as u64);
        crate::idt::set_idt_gate(0x80, crate::interrupts::syscall_handler_stub as u64);
        crate::idt::set_idt_gate(0x0D, crate::interrupts::gp_fault_handler_stub as u64);
        crate::idt::set_idt_gate(0x0E, crate::interrupts::page_fault_handler_stub as u64);
        crate::idt::set_idt_gate(0x06, crate::interrupts::invalid_opcode_handler_stub as u64);
        crate::interrupts::init_syscalls();
    }
    unsafe {
        let divisor = 1193180 / 100;
        crate::serial::outb(0x43, 0x36);
        crate::serial::outb(0x40, (divisor & 0xFF) as u8);
        crate::serial::outb(0x40, ((divisor >> 8) & 0xFF) as u8);
    }
    print_serial(b"OK\n");
}
fn init_memory() {
    print_serial(b"[INIT] Memory (PMM)... ");
    crate::pmm::init();
    let frame = crate::pmm::allocate_frame();
    if frame > 0 {
        print_serial(b"OK\n");
    } else {
        print_serial(b"FAIL\n");
    }
}
fn init_tasks() {
    print_serial(b"[INIT] Tasks... ");
    crate::task::init();
    print_serial(b"OK\n");
}
fn init_drivers() {
    print_serial(b"[INIT] Drivers (E1000)... ");
    unsafe {
        crate::drivers::e1000::init_e1000();
    }
    crate::task::spawn(crate::net::net_task);
    print_serial(b"OK\n");
}
fn init_fs() {
    print_serial(b"[INIT] Filesystem... ");
    crate::fs::init();
    print_serial(b"OK\n");
}
fn init_shell() {
    print_serial(b"[INIT] Shell... ");
    crate::shell::init_shell();
    print_serial(b"OK\n");
}
fn enable_interrupts() {
    print_serial(b"[INIT] Enabling Interrupts... ");
    unsafe { asm!("sti") };
    print_serial(b"OK\n");
}
struct SerialWriter;
impl Write for SerialWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        print_serial(s.as_bytes());
        Ok(())
    }
}
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print_serial(b"\nPANIC DETECTED!\n");
    let mut writer = SerialWriter;
    let _ = write!(writer, "{}", info);
    loop {}
}
