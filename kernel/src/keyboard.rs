static mut SHIFT_PRESSED: bool = false;
static mut E0_PREFIX: bool = false;
pub const KEY_PAGE_UP: u8 = 0xF1;
pub const KEY_PAGE_DOWN: u8 = 0xF2;
pub fn scancode_to_char(scancode: u8) -> u8 {
    unsafe {
        if scancode == 0xE0 {
            E0_PREFIX = true;
            return 0;
        }
        if E0_PREFIX {
            E0_PREFIX = false;
            match scancode {
                0x49 => return KEY_PAGE_UP,
                0x51 => return KEY_PAGE_DOWN,
                0x53 => return 0x08,
                _ => return 0,
            }
        }
        match scancode {
            0x2A | 0x36 => {
                SHIFT_PRESSED = true;
                0
            }
            0xAA | 0xB6 => {
                SHIFT_PRESSED = false;
                0
            }
            0x0E => 0x08,
            0x39 => b' ',
            0x1C => b'\n',
            0x02 => {
                if SHIFT_PRESSED {
                    b'!'
                } else {
                    b'1'
                }
            }
            0x03 => {
                if SHIFT_PRESSED {
                    b'"'
                } else {
                    b'2'
                }
            }
            0x04 => {
                if SHIFT_PRESSED {
                    b'3'
                } else {
                    b'3'
                }
            }
            0x05 => {
                if SHIFT_PRESSED {
                    b'$'
                } else {
                    b'4'
                }
            }
            0x06 => {
                if SHIFT_PRESSED {
                    b'%'
                } else {
                    b'5'
                }
            }
            0x07 => {
                if SHIFT_PRESSED {
                    b'&'
                } else {
                    b'6'
                }
            }
            0x08 => {
                if SHIFT_PRESSED {
                    b'/'
                } else {
                    b'7'
                }
            }
            0x09 => {
                if SHIFT_PRESSED {
                    b'('
                } else {
                    b'8'
                }
            }
            0x0A => {
                if SHIFT_PRESSED {
                    b')'
                } else {
                    b'9'
                }
            }
            0x0B => {
                if SHIFT_PRESSED {
                    b'='
                } else {
                    b'0'
                }
            }
            0x0C => {
                if SHIFT_PRESSED {
                    b'?'
                } else {
                    b'\''
                }
            }
            0x0D => {
                if SHIFT_PRESSED {
                    b'^'
                } else {
                    b'i'
                }
            }
            0x10 => {
                if SHIFT_PRESSED {
                    b'Q'
                } else {
                    b'q'
                }
            }
            0x11 => {
                if SHIFT_PRESSED {
                    b'W'
                } else {
                    b'w'
                }
            }
            0x12 => {
                if SHIFT_PRESSED {
                    b'E'
                } else {
                    b'e'
                }
            }
            0x13 => {
                if SHIFT_PRESSED {
                    b'R'
                } else {
                    b'r'
                }
            }
            0x14 => {
                if SHIFT_PRESSED {
                    b'T'
                } else {
                    b't'
                }
            }
            0x15 => {
                if SHIFT_PRESSED {
                    b'Y'
                } else {
                    b'y'
                }
            }
            0x16 => {
                if SHIFT_PRESSED {
                    b'U'
                } else {
                    b'u'
                }
            }
            0x17 => {
                if SHIFT_PRESSED {
                    b'I'
                } else {
                    b'i'
                }
            }
            0x18 => {
                if SHIFT_PRESSED {
                    b'O'
                } else {
                    b'o'
                }
            }
            0x19 => {
                if SHIFT_PRESSED {
                    b'P'
                } else {
                    b'p'
                }
            }
            0x1A => {
                if SHIFT_PRESSED {
                    b'e'
                } else {
                    b'e'
                }
            }
            0x1B => {
                if SHIFT_PRESSED {
                    b'+'
                } else {
                    b'*'
                }
            }
            0x1E => {
                if SHIFT_PRESSED {
                    b'A'
                } else {
                    b'a'
                }
            }
            0x1F => {
                if SHIFT_PRESSED {
                    b'S'
                } else {
                    b's'
                }
            }
            0x20 => {
                if SHIFT_PRESSED {
                    b'D'
                } else {
                    b'd'
                }
            }
            0x21 => {
                if SHIFT_PRESSED {
                    b'F'
                } else {
                    b'f'
                }
            }
            0x22 => {
                if SHIFT_PRESSED {
                    b'G'
                } else {
                    b'g'
                }
            }
            0x23 => {
                if SHIFT_PRESSED {
                    b'H'
                } else {
                    b'h'
                }
            }
            0x24 => {
                if SHIFT_PRESSED {
                    b'J'
                } else {
                    b'j'
                }
            }
            0x25 => {
                if SHIFT_PRESSED {
                    b'K'
                } else {
                    b'k'
                }
            }
            0x26 => {
                if SHIFT_PRESSED {
                    b'L'
                } else {
                    b'l'
                }
            }
            0x27 => {
                if SHIFT_PRESSED {
                    b'o'
                } else {
                    b'o'
                }
            }
            0x28 => {
                if SHIFT_PRESSED {
                    b'a'
                } else {
                    b'a'
                }
            }
            0x29 => {
                if SHIFT_PRESSED {
                    b'|'
                } else {
                    b'\\'
                }
            }
            0x2C => {
                if SHIFT_PRESSED {
                    b'Z'
                } else {
                    b'z'
                }
            }
            0x2D => {
                if SHIFT_PRESSED {
                    b'X'
                } else {
                    b'x'
                }
            }
            0x2E => {
                if SHIFT_PRESSED {
                    b'C'
                } else {
                    b'c'
                }
            }
            0x2F => {
                if SHIFT_PRESSED {
                    b'V'
                } else {
                    b'v'
                }
            }
            0x30 => {
                if SHIFT_PRESSED {
                    b'B'
                } else {
                    b'b'
                }
            }
            0x31 => {
                if SHIFT_PRESSED {
                    b'N'
                } else {
                    b'n'
                }
            }
            0x32 => {
                if SHIFT_PRESSED {
                    b'M'
                } else {
                    b'm'
                }
            }
            0x33 => {
                if SHIFT_PRESSED {
                    b';'
                } else {
                    b','
                }
            }
            0x34 => {
                if SHIFT_PRESSED {
                    b':'
                } else {
                    b'.'
                }
            }
            0x35 => {
                if SHIFT_PRESSED {
                    b'_'
                } else {
                    b'-'
                }
            }
            _ => 0,
        }
    }
}
