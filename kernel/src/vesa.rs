#[repr(C, packed)]
pub struct VesaInfo {
    pub framebuffer_addr: u64,
    pub width: u16,
    pub height: u16,
    pub pitch: u16,
    pub bpp: u16,
}
static mut VESA_INFO: VesaInfo = VesaInfo {
    framebuffer_addr: 0,
    width: 0,
    height: 0,
    pitch: 0,
    bpp: 0,
};
const FONT_WIDTH: usize = 8;
const FONT_HEIGHT: usize = 16;
static FONT_8X8: [u8; 128 * 8] = [0; 1024];
pub fn put_pixel(x: u16, y: u16, color: u32) {
    unsafe {
        if x >= VESA_INFO.width || y >= VESA_INFO.height {
            return;
        }
        let offset = (y as u64 * VESA_INFO.pitch as u64) + (x as u64 * (VESA_INFO.bpp as u64 / 8));
        let addr = (VESA_INFO.framebuffer_addr + offset) as *mut u8;
        if VESA_INFO.bpp == 32 {
            addr.write_volatile((color & 0xFF) as u8);
            addr.add(1).write_volatile(((color >> 8) & 0xFF) as u8);
            addr.add(2).write_volatile(((color >> 16) & 0xFF) as u8);
            addr.add(3).write_volatile(0xFF);
        }
    }
}
pub fn init() {
    unsafe {
        let info_ptr = 0x0800 as *const VesaInfo;
        VESA_INFO = core::ptr::read_unaligned(info_ptr);
        crate::serial::print_serial(b"[VESA] Init. Width: ");
        crate::serial::print_u64(VESA_INFO.width as u64);
        crate::serial::print_serial(b" Height: ");
        crate::serial::print_u64(VESA_INFO.height as u64);
        crate::serial::print_serial(b" FB: ");
        crate::serial::print_hex(VESA_INFO.framebuffer_addr);
        crate::serial::print_serial(b"\n");
        if VESA_INFO.width > 0 {
            for y in 0..50 {
                for x in 0..50 {
                    put_pixel(x, y, 0xFFFFFF);
                }
            }
            for y in 0..50 {
                for x in 50..100 {
                    put_pixel(x, y, 0xFF0000);
                }
            }
            for y in 0..50 {
                for x in 100..150 {
                    put_pixel(x, y, 0x00FF00);
                }
            }
        }
    }
}
static mut INPUT_START_X: u16 = 0;
static mut INPUT_START_Y: u16 = 0;
pub fn set_input_start_pos() {
    unsafe {
        INPUT_START_X = CURSOR_X;
        INPUT_START_Y = CURSOR_Y;
    }
}
pub fn clear_screen(color: u32) {
    unsafe {
        let size = (VESA_INFO.height as usize) * (VESA_INFO.pitch as usize);
        let ptr = VESA_INFO.framebuffer_addr as *mut u8;
        if color == 0 {
            core::ptr::write_bytes(ptr, 0, size);
        } else {
            for y in 0..VESA_INFO.height {
                for x in 0..VESA_INFO.width {
                    put_pixel(x, y, color);
                }
            }
        }
    }
}
pub static mut ZOOM_NUM: usize = 2;
pub static mut ZOOM_DEN: usize = 1;
pub fn set_zoom(num: usize, den: usize) {
    unsafe {
        if num == 0 || den == 0 {
            return;
        }
        ZOOM_NUM = num;
        ZOOM_DEN = den;
    }
}
pub fn draw_char(x: u16, y: u16, c: u8, color: u32) {
    unsafe {
        let width = (8 * ZOOM_NUM) / ZOOM_DEN;
        let height = (8 * ZOOM_NUM) / ZOOM_DEN;
        if width == 0 || height == 0 {
            return;
        }
        let font_offset = c as usize * 8;
        for dy in 0..height {
            let src_y = (dy * ZOOM_DEN) / ZOOM_NUM;
            if src_y >= 8 {
                continue;
            }
            let row = crate::font::FONT[font_offset + src_y];
            for dx in 0..width {
                let src_x = (dx * ZOOM_DEN) / ZOOM_NUM;
                if src_x >= 8 {
                    continue;
                }
                let pixel_color = if (row >> (7 - src_x)) & 1 == 1 {
                    color
                } else {
                    0x000000
                };
                put_pixel(x + dx as u16, y + dy as u16, pixel_color);
            }
        }
    }
}
pub static mut CURSOR_X: u16 = 0;
pub static mut CURSOR_Y: u16 = 0;
pub fn scroll_up() {
    unsafe {
        let line_h = ((16 * ZOOM_NUM) / ZOOM_DEN) as usize;
        let pitch = VESA_INFO.pitch as usize;
        let height = VESA_INFO.height as usize;
        if line_h == 0 {
            return;
        }
        let fb = VESA_INFO.framebuffer_addr as *mut u8;
        let copy_size = (height - line_h) * pitch;
        core::ptr::copy(fb.add(line_h * pitch), fb, copy_size);
        let bottom_start = fb.add(copy_size);
        core::ptr::write_bytes(bottom_start, 0, line_h * pitch);
    }
}
pub fn print_str(msg: &[u8], color: u32) {
    unsafe {
        let char_w = (8 * ZOOM_NUM) / ZOOM_DEN;
        let line_h = (16 * ZOOM_NUM) / ZOOM_DEN;
        if char_w == 0 || line_h == 0 {
            return;
        }
        for &byte in msg {
            if byte == b'\n' {
                CURSOR_X = 0;
                CURSOR_Y += line_h as u16;
            } else if byte == 0x08 {
                let can_delete = if CURSOR_Y > INPUT_START_Y {
                    true
                } else if CURSOR_Y == INPUT_START_Y {
                    CURSOR_X > INPUT_START_X
                } else {
                    false
                };
                if can_delete && CURSOR_X >= char_w as u16 {
                    CURSOR_X -= char_w as u16;
                    draw_char(CURSOR_X, CURSOR_Y, b' ', 0);
                }
            } else {
                draw_char(CURSOR_X, CURSOR_Y, byte, color);
                CURSOR_X += char_w as u16;
            }
            if CURSOR_X >= VESA_INFO.width - char_w as u16 {
                CURSOR_X = 0;
                CURSOR_Y += line_h as u16;
            }
            if CURSOR_Y >= VESA_INFO.height - line_h as u16 {
                scroll_up();
                CURSOR_Y -= line_h as u16;
                if INPUT_START_Y >= line_h as u16 {
                    INPUT_START_Y -= line_h as u16;
                } else {
                    INPUT_START_Y = 0;
                }
            }
        }
    }
}
