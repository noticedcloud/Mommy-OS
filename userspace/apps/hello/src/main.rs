#![no_std]
#![no_main]
use mommylib::{exit, print};
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    exit(1);
}
#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    print("Hello from Userspace!\n");
    exit(0);
}
