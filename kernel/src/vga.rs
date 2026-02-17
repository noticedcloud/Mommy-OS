pub static mut CURRENT_COLOR: u32 = 0x00FF00;
pub fn init() {
    crate::vesa::init();
    crate::serial::print_serial(b"VESA Init Done via VGA wrapper\n");
}
pub fn set_color(c: u8) {
    unsafe {
        CURRENT_COLOR = vga_to_rgb(c);
    }
}
fn vga_to_rgb(c: u8) -> u32 {
    match c {
        0x00 => 0x000000,
        0x01 => 0x0000AA,
        0x02 => 0x00AA00,
        0x03 => 0x00AAAA,
        0x04 => 0xAA0000,
        0x05 => 0xAA00AA,
        0x06 => 0xAA5500,
        0x07 => 0xAAAAAA,
        0x08 => 0x555555,
        0x09 => 0x5555FF,
        0x0A => 0x00FF00,
        0x0B => 0x00FFFF,
        0x0C => 0xFF5555,
        0x0D => 0xFF55FF,
        0x0E => 0xFFFF55,
        0x0F => 0xFFFFFF,
        _ => 0xFFFFFF,
    }
}
pub fn clear_screen() {
    crate::vesa::clear_screen(0x000000);
    unsafe {
        crate::vesa::CURSOR_X = 0;
        crate::vesa::CURSOR_Y = 0;
    }
}
pub fn set_input_start() {
    crate::vesa::set_input_start_pos();
}
pub fn backspace() {
    print(b"\x08");
}
pub fn scroll_up() {}
pub fn scroll_down() {}
pub fn flush() {}
pub fn snap_to_cursor() {}
pub fn update_hardware_cursor() {}
pub fn enable_cursor(_start: u8, _end: u8) {}
pub fn print(message: &[u8]) {
    unsafe {
        crate::vesa::print_str(message, CURRENT_COLOR);
    }
}
pub fn print_u64_vga(val: u64) {
    let mut buffer = [b'0'; 18];
    buffer[0] = b'0';
    buffer[1] = b'x';
    for i in 0..16 {
        let nibble = ((val >> ((15 - i) * 4)) & 0xF) as u8;
        buffer[i + 2] = if nibble < 10 {
            b'0' + nibble
        } else {
            b'A' + (nibble - 10)
        };
    }
    print(&buffer);
}
