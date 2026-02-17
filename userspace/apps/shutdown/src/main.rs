#![no_std]
#![no_main]
use mommylib::{exit, shutdown};
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    shutdown();
    exit(0);
}
