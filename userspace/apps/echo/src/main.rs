#![no_std]
#![no_main]
use core::arch::asm;

#[panic_handler]

fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
#[inline(always)]
unsafe fn print_bytes(ptr: *const u8, len: usize) {
    asm!(
        "int 0x80",
        in("rax") 1,
        in("rdi") ptr,
        in("rsi") len,
    );
}

#[inline(always)]
unsafe fn exit(code: i32) -> ! {
    asm!(
        "int 0x80",
        in("rax") 0,
        in("rdi") code,
        options(noreturn)
    );
}
core::arch::global_asm!(
    ".section .text.entry",
    ".global _start",
    "_start:",
    "mov rdi, rsp",
    "call main",
    "ud2",
);

#[no_mangle]
pub extern "C" fn main(args_ptr: *const u8) -> ! {
    unsafe {
        let mut len = 0;
        while core::ptr::read_volatile(args_ptr.add(len)) != 0 {
            len += 1;
        }

        if len > 0 {
            print_bytes(args_ptr, len);
        }

        print_bytes(b"\n".as_ptr(), 1);
        exit(0);
    }
}
