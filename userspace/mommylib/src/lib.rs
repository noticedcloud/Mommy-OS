#![no_std]
use core::arch::asm;
#[inline(always)]
pub fn print(msg: &str) {
    unsafe {
        asm!(
            "int 0x80",
            in("rax") 1,
            in("rdi") msg.as_ptr(),
            in("rsi") msg.len(),
        );
    }
}
#[inline(always)]
pub fn exit(code: i32) -> ! {
    unsafe {
        asm!(
            "int 0x80",
            in("rax") 0,
            in("rdi") code,
            options(noreturn)
        );
    }
}
#[inline(always)]
pub fn clear() {
    unsafe {
        asm!(
            "int 0x80",
            in("rax") 2,
        );
    }
}
#[inline(always)]
pub fn shutdown() {
    unsafe {
        asm!(
            "int 0x80",
            in("rax") 3,
        );
    }
}
#[inline(always)]
pub fn reboot() {
    unsafe {
        asm!(
            "int 0x80",
            in("rax") 4,
        );
    }
}
#[inline(always)]
pub fn read_dir(path: &str, index: usize, out_buf: &mut [u8]) -> i32 {
    let res: i32;
    unsafe {
        asm!(
            "int 0x80",
            in("rax") 5,
            in("rdi") path.as_ptr(),
            in("rsi") path.len(),
            in("rdx") index,
            in("r10") out_buf.as_mut_ptr(),
            lateout("rax") res,
        );
    }
    res
}
#[inline(always)]
pub fn set_color(color: u8) {
    unsafe {
        asm!(
            "int 0x80",
            in("rax") 6,
            in("rdi") color as u64,
        );
    }
}
