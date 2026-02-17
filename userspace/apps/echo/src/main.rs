#![no_std]
#![no_main]
use core::arch::asm;
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
unsafe fn print_bytes(ptr: *const u8, len: usize) {
    asm!(
        "int 0x80",
        in("rax") 1,
        in("rdi") ptr,
        in("rsi") len,
    );
}
unsafe fn exit(code: i32) -> ! {
    asm!(
        "int 0x80",
        in("rax") 0,
        in("rdi") code,
        options(noreturn)
    );
}
#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start(argc: usize, argv: *const u8) -> ! {
    if argc > 0 && !argv.is_null() {
        unsafe {
            print_bytes(argv, argc);
        }
    }
    unsafe {
        print_bytes(b"\n".as_ptr(), 1);
        exit(0);
    }
}
