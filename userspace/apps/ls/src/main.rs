#![no_std]
#![no_main]
use mommylib::{exit, print, read_dir};
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    let mut buffer = [0u8; 32];
    let mut index = 0;
    print("LS Start\n");
    let mut path = [b'.', 0];
    loop {
        let res = read_dir(
            unsafe { core::str::from_utf8_unchecked(&path[..1]) },
            index,
            &mut buffer,
        );
        if res == 1 {
            break;
        }
        let mut len = 32;
        for k in 0..32 {
            if buffer[k] == 0 {
                len = k;
                break;
            }
        }
        let name_str = unsafe { core::str::from_utf8_unchecked(&buffer[0..len]) };
        if res == 2 {
            mommylib::set_color(0x0B);
            print(name_str);
            mommylib::set_color(0x0B);
            print("/");
            mommylib::set_color(0x0A);
        } else {
            print(name_str);
        }
        print("  ");
        index += 1;
    }
    print("\n");
    exit(0);
}
