use core::arch::asm;
#[inline(always)]
pub fn outb(port: u16, val: u8) {
    unsafe {
        asm!("out dx, al", in("dx") port, in("al") val, options(nomem, nostack, preserves_flags));
    }
}
#[inline(always)]
pub fn inb(port: u16) -> u8 {
    let res: u8;
    unsafe {
        asm!("in al, dx", out("al") res, in("dx") port, options(nomem, nostack, preserves_flags));
    }
    res
}
#[inline(always)]
pub fn outw(port: u16, val: u16) {
    unsafe {
        asm!("out dx, ax", in("dx") port, in("ax") val, options(nomem, nostack, preserves_flags));
    }
}
pub fn print_serial(message: &[u8]) {
    for &byte in message {
        while (inb(0x3FD) & 0x20) == 0 {}
        outb(0x3F8, byte);
    }
}
pub fn print_hex(mut n: u64) {
    let hex = b"0123456789ABCDEF";
    if n == 0 {
        print_serial(b"0x0");
        return;
    }
    print_serial(b"0x");
    let mut buffer = [0u8; 16];
    let mut i = 0;
    while n > 0 {
        buffer[i] = hex[(n & 0xF) as usize];
        n >>= 4;
        i += 1;
    }
    while i > 0 {
        i -= 1;
        outb(0x3F8, buffer[i]);
    }
}
pub unsafe fn print_dec(mut n: u64) {
    if n == 0 {
        print_serial(b"0");
        return;
    }
    let mut buffer = [0u8; 20];
    let mut i = 0;
    while n > 0 {
        buffer[i] = (n % 10) as u8 + b'0';
        n /= 10;
        i += 1;
    }
    while i > 0 {
        i -= 1;
        outb(0x3F8, buffer[i]);
    }
}
pub fn print_u64(val: u64) {
    unsafe {
        print_dec(val);
    }
}
