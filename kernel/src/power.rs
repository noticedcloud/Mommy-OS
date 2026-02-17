use crate::serial::{outb, outw};
use crate::vga::print;
pub fn shutdown() {
    print(b"Shutting down...\n");
    outw(0x604, 0x2000);
    outw(0x4004, 0x3400);
    outb(0xB004, 0x20);
    print(b"Shutdown failed. Please power off manually.\n");
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
pub fn reboot() {
    print(b"Rebooting...\n");
    let mut good: u8 = 0x02;
    while (good & 0x02) != 0 {
        good = crate::serial::inb(0x64);
    }
    outb(0x64, 0xFE);
    print(b"Reboot failed. Please reset manually.\n");
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
