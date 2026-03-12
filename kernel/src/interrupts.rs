use crate::keyboard::scancode_to_char;
use crate::serial::{inb, outb};
use core::arch::global_asm;

global_asm!(
    ".extern generic_handler",
    ".global generic_handler_stub",
    "generic_handler_stub:",
    "push rax",
    "push rcx",
    "push rdx",
    "push rsi",
    "push rdi",
    "push r8",
    "push r9",
    "push r10",
    "push r11",
    "call generic_handler",
    "pop r11",
    "pop r10",
    "pop r9",
    "pop r8",
    "pop rdi",
    "pop rsi",
    "pop rdx",
    "pop rcx",
    "pop rax",
    "iretq",
    ".extern keyboard_handler",
    ".global keyboard_handler_stub",
    "keyboard_handler_stub:",
    "push rax",
    "push rcx",
    "push rdx",
    "push rsi",
    "push rdi",
    "push r8",
    "push r9",
    "push r10",
    "push r11",
    "call keyboard_handler",
    "pop r11",
    "pop r10",
    "pop r9",
    "pop r8",
    "pop rdi",
    "pop rsi",
    "pop rdx",
    "pop rcx",
    "pop rax",
    "iretq",
    ".extern timer_handler",
    ".global timer_handler_stub",
    "timer_handler_stub:",
    "push rax",
    "push rcx",
    "push rdx",
    "push rsi",
    "push rdi",
    "push r8",
    "push r9",
    "push r10",
    "push r11",
    "call timer_handler",
    "pop r11",
    "pop r10",
    "pop r9",
    "pop r8",
    "pop rdi",
    "pop rsi",
    "pop rdx",
    "pop rcx",
    "pop rax",
    "iretq",
    ".extern syscall_dispatcher"
);

#[repr(C)]

pub struct CpuLocal {
    pub kernel_stack: u64,
    pub user_stack: u64,
}
pub static mut CPU_LOCAL: CpuLocal = CpuLocal {
    kernel_stack: 0,
    user_stack: 0,
};

global_asm!(
    ".extern gp_fault_handler",
    ".global long_mode_syscall_handler_stub",
    "long_mode_syscall_handler_stub:",
    "swapgs",
    "mov gs:[8], rsp",
    "mov rsp, gs:[0]",
    "push qword ptr gs:[8]",
    "push r11",
    "push rcx",
    "push rax",
    "push rdx",
    "push rsi",
    "push rdi",
    "push r8",
    "push r9",
    "push r10",
    "mov r9, r8",
    "mov r8, r10",
    "mov rcx, rdx",
    "mov rdx, rsi",
    "mov rsi, rdi",
    "mov rdi, rax",
    "call syscall_dispatcher",
    "pop r10",
    "pop r9",
    "pop r8",
    "pop rdi",
    "pop rsi",
    "pop rdx",
    "add rsp, 8",
    "pop rcx",
    "pop r11",
    "pop rsp",
    "swapgs",
    "sysretq",
    "jmp rcx",
    ".global syscall_handler_stub",
    "syscall_handler_stub:",
    "push rax",
    "push rcx",
    "push rdx",
    "push rsi",
    "push rdi",
    "push r8",
    "push r9",
    "push r10",
    "push r11",
    "mov r9, r8",
    "mov r8, r10",
    "mov rcx, rdx",
    "mov rdx, rsi",
    "mov rsi, rdi",
    "mov rdi, rax",
    "call syscall_dispatcher",
    "pop r11",
    "pop r10",
    "pop r9",
    "pop r8",
    "pop rdi",
    "pop rsi",
    "pop rdx",
    "pop rcx",
    "add rsp, 8",
    "iretq",
    ".extern e1000_handler",
    ".global e1000_handler_stub",
    "e1000_handler_stub:",
    "push rax",
    "push rcx",
    "push rdx",
    "push rsi",
    "push rdi",
    "push r8",
    "push r9",
    "push r10",
    "push r11",
    "call e1000_handler",
    "mov al, 0x20",
    "out 0x20, al",
    "out 0xA0, al",
    "pop r11",
    "pop r10",
    "pop r9",
    "pop r8",
    "pop rdi",
    "pop rsi",
    "pop rdx",
    "pop rcx",
    "pop rax",
    "iretq"
);
extern "C" {

    pub fn generic_handler_stub();

    pub fn keyboard_handler_stub();

    pub fn timer_handler_stub();

    pub fn syscall_handler_stub();

    pub fn long_mode_syscall_handler_stub();

    pub fn gp_fault_handler_stub();

    pub fn page_fault_handler_stub();

    pub fn invalid_opcode_handler_stub();

    pub fn e1000_handler_stub();
}
pub fn init_syscalls() {
    unsafe {
        let efer_msr: u32 = 0xC0000080;
        let mut low: u32;
        let mut high: u32;
        core::arch::asm!("rdmsr", in("ecx") efer_msr, out("eax") low, out("edx") high, options(nostack));
        low |= 1;
        core::arch::asm!("wrmsr", in("ecx") efer_msr, in("eax") low, in("edx") high, options(nostack));
        let star: u64 = (0x0008 << 32) | (0x0018 << 48);
        let star_low = star as u32;
        let star_high = (star >> 32) as u32;
        let star_msr: u32 = 0xC0000081;
        core::arch::asm!("wrmsr", in("ecx") star_msr, in("eax") star_low, in("edx") star_high, options(nostack));
        let handler_addr = long_mode_syscall_handler_stub as *const () as u64;
        let handler_low = handler_addr as u32;
        let handler_high = (handler_addr >> 32) as u32;
        let lstar_msr: u32 = 0xC0000082;
        core::arch::asm!("wrmsr", in("ecx") lstar_msr, in("eax") handler_low, in("edx") handler_high, options(nostack));

        let kernel_gs_base_msr: u32 = 0xC0000102;
        let cpu_local_addr = &raw const CPU_LOCAL as u64;
        let cpu_local_low = cpu_local_addr as u32;
        let cpu_local_high = (cpu_local_addr >> 32) as u32;
        core::arch::asm!("wrmsr", in("ecx") kernel_gs_base_msr, in("eax") cpu_local_low, in("edx") cpu_local_high, options(nostack));

        let mask: u64 = 0x200;
        let mask_low = mask as u32;
        let mask_high = (mask >> 32) as u32;
        let sfmask_msr: u32 = 0xC0000084;
        core::arch::asm!("wrmsr", in("ecx") sfmask_msr, in("eax") mask_low, in("edx") mask_high, options(nostack));
        let mut check_low: u32;
        let mut check_high: u32;
        core::arch::asm!("rdmsr", in("ecx") lstar_msr, out("eax") check_low, out("edx") check_high, options(nostack));
        let check_val = ((check_high as u64) << 32) | (check_low as u64);
        crate::serial::print_serial(b"LSTAR Set To: ");
        crate::serial::print_hex(check_val);
        crate::serial::print_serial(b"\n");
        crate::serial::print_serial(b"Stub Address: ");
        crate::serial::print_hex(handler_addr);
        crate::serial::print_serial(b"\n");
        crate::serial::print_serial(b"[KERNEL] Syscalls Initialized (MSRs set).\n");
    }
}
#[no_mangle]
pub extern "C" fn generic_handler() {
    outb(0x20, 0x20);
    outb(0xA0, 0x20);
}
global_asm!(
    ".extern gp_fault_handler",
    ".global gp_fault_handler_stub",
    "gp_fault_handler_stub:",
    "push rax",
    "push rcx",
    "push rdx",
    "push rsi",
    "push rdi",
    "push r8",
    "push r9",
    "push r10",
    "push r11",
    "mov rdi, rsp",
    "call gp_fault_handler",
    "1:",
    "hlt",
    "jmp 1b",
    ".extern page_fault_handler",
    ".global page_fault_handler_stub",
    "page_fault_handler_stub:",
    "push rax",
    "push rcx",
    "push rdx",
    "push rsi",
    "push rdi",
    "push r8",
    "push r9",
    "push r10",
    "push r11",
    "mov rdi, rsp",
    "call page_fault_handler",
    "1:",
    "hlt",
    "jmp 1b",
    ".extern invalid_opcode_handler",
    ".global invalid_opcode_handler_stub",
    "invalid_opcode_handler_stub:",
    "push rax",
    "push rcx",
    "push rdx",
    "push rsi",
    "push rdi",
    "push r8",
    "push r9",
    "push r10",
    "push r11",
    "mov rdi, rsp",
    "call invalid_opcode_handler",
    "1:",
    "hlt",
    "jmp 1b",
);
#[repr(C)]

#[derive(Debug)]

pub struct ExceptionStackFrame {
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rax: u64,
    pub error_code: u64,
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}
#[no_mangle]
pub extern "C" fn gp_fault_handler(stack_frame: *const ExceptionStackFrame) {
    unsafe {
        let mut old_cr3: u64;
        core::arch::asm!("mov {}, cr3", out(reg) old_cr3);
        let kernel_cr3 = crate::task::get_kernel_cr3();
        if old_cr3 != kernel_cr3 {
            core::arch::asm!("mov cr3, {}", in(reg) kernel_cr3);
        }
        let frame = &*stack_frame;
        crate::serial::print_serial(b"\n[MOM PANIC] GP FAULT (0xD)!\n");
        crate::serial::print_serial(b"RIP: ");
        crate::serial::print_hex(frame.rip);
        crate::serial::print_serial(b"\nError Code: ");
        crate::serial::print_hex(frame.error_code);
        crate::serial::print_serial(b"\nRSP: ");
        crate::serial::print_hex(frame.rsp);
        crate::serial::print_serial(b"\nCS: ");
        crate::serial::print_hex(frame.cs);
        crate::serial::print_serial(b"\nRSI: ");
        crate::serial::print_hex(frame.rsi);
        crate::serial::print_serial(b"\nRDI: ");
        crate::serial::print_hex(frame.rdi);
        crate::serial::print_serial(b"\nRAX: ");
        crate::serial::print_hex(frame.rax);
        crate::serial::print_serial(b"\nCR3 (at fault): ");
        crate::serial::print_hex(old_cr3);
        crate::serial::print_serial(b"\nKernel CR3: ");
        crate::serial::print_hex(kernel_cr3);
        crate::serial::print_serial(b"\n");
        crate::vga::print(b"\nGP FAULT (0xD)! RIP: ");
        crate::vga::print_u64_vga(frame.rip);
        crate::vga::print(b"\n");
    }
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
#[no_mangle]
pub extern "C" fn page_fault_handler(stack_frame: *const ExceptionStackFrame) {
    unsafe {
        let cr2: u64;
        core::arch::asm!("mov {}, cr2", out(reg) cr2);
        let mut old_cr3: u64;
        core::arch::asm!("mov {}, cr3", out(reg) old_cr3);
        let kernel_cr3 = crate::task::get_kernel_cr3();
        if old_cr3 != kernel_cr3 {
            core::arch::asm!("mov cr3, {}", in(reg) kernel_cr3);
        }
        let frame = &*stack_frame;
        crate::serial::print_serial(b"\n[MOM PANIC] PAGE FAULT (0xE)!\n");
        crate::serial::print_serial(b"CR2: ");
        crate::serial::print_hex(cr2);
        crate::serial::print_serial(b"\nRIP: ");
        crate::serial::print_hex(frame.rip);
        crate::serial::print_serial(b"\nError Code: ");
        crate::serial::print_hex(frame.error_code);
        crate::serial::print_serial(b"\nRSP: ");
        crate::serial::print_hex(frame.rsp);
        crate::serial::print_serial(b"\nRSI: ");
        crate::serial::print_hex(frame.rsi);
        crate::serial::print_serial(b"\nRDI: ");
        crate::serial::print_hex(frame.rdi);
        crate::serial::print_serial(b"\nCR3 (at fault): ");
        crate::serial::print_hex(old_cr3);
        crate::serial::print_serial(b"\n");
        crate::vga::print(b"\nPAGE FAULT! CR2: ");
        crate::vga::print_u64_vga(cr2);
        crate::vga::print(b"\n");
    }
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
#[no_mangle]
pub extern "C" fn keyboard_handler() {
    let scancode = inb(0x60);
    let c = scancode_to_char(scancode);
    if c != 0 {
        if c == crate::keyboard::KEY_PAGE_UP {
            crate::vga::scroll_up();
        } else if c == crate::keyboard::KEY_PAGE_DOWN {
            crate::vga::scroll_down();
        } else {
            crate::shell::handle_key(c);
        }
    }
    outb(0x20, 0x20);
}
pub static mut TICKS: u64 = 0;

#[no_mangle]
pub extern "C" fn timer_handler() {
    outb(0x20, 0x20);
    unsafe {
        TICKS += 1;
        if TICKS % 100 == 0 {
            crate::serial::print_serial(b".");
        }
    }
    crate::task::schedule();
}
#[no_mangle]
pub extern "C" fn invalid_opcode_handler(stack_frame: *const ExceptionStackFrame) {
    unsafe {
        crate::vga::print(b"\nINVALID OPCODE (0x6)!\n");
        let frame = &*stack_frame;
        crate::vga::print(b"RIP: ");
        crate::vga::print_u64_vga(frame.rip);
        crate::vga::print(b"\n");
        crate::serial::print_serial(b"\n MOM PANIC: EXCEPTION: INVALID OPCODE(COME ON BABY, YOU CAN DO BETTER THEN THAT, CAN'T YOU?)\n");
        crate::serial::print_serial(b"RIP: ");
        crate::serial::print_hex(frame.rip);
        crate::serial::print_serial(b"\n");
    }
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

#[no_mangle]
pub extern "C" fn e1000_handler() {
    unsafe {
        crate::drivers::e1000::handle_interrupt();
        outb(0x20, 0x20);
        outb(0xA0, 0x20);
    }
}
#[inline(always)]

pub fn without_interrupts<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let flags: u64;
    unsafe {
        core::arch::asm!("pushfq", "pop {}", out(reg) flags);
        core::arch::asm!("cli");
    }
    let ret = f();
    if (flags & 0x200) != 0 {
        unsafe { core::arch::asm!("sti"); }
    }
    ret
}
