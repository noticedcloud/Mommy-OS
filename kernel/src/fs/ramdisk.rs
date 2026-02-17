use crate::pmm::allocate_frame;
pub const BLOCK_SIZE: usize = 4096;
pub const MAX_BLOCKS: usize = 1024;
static mut BLOCK_BITMAP: [u8; MAX_BLOCKS / 8] = [0; MAX_BLOCKS / 8];
static mut BLOCK_STORAGE: [usize; MAX_BLOCKS] = [0; MAX_BLOCKS];
pub fn init_ramdisk() {
    unsafe {
        for i in 0..MAX_BLOCKS {
            let frame = allocate_frame();
            if frame == 0 {
                crate::serial::print_serial(b"[FS] RAMDISK ALLOC FAILED (OOM) at block ");
                crate::serial::print_hex(i as u64);
                crate::serial::print_serial(b"\n");
                return;
            }
            BLOCK_STORAGE[i] = frame as usize;
        }
        for i in 0..(MAX_BLOCKS / 8) {
            BLOCK_BITMAP[i] = 0;
        }
        crate::serial::print_serial(b"[FS] RAMDISK Init OK (1024 blocks allocated)\n");
    }
}
pub fn read_block(block_index: usize, buffer: &mut [u8]) {
    unsafe {
        if block_index >= MAX_BLOCKS {
            return;
        }
        let ptr = BLOCK_STORAGE[block_index] as *const u8;
        for i in 0..BLOCK_SIZE {
            buffer[i] = *ptr.add(i);
        }
    }
}
pub fn write_block(block_index: usize, buffer: &[u8]) {
    unsafe {
        if block_index >= MAX_BLOCKS {
            return;
        }
        let ptr = BLOCK_STORAGE[block_index] as *mut u8;
        for i in 0..BLOCK_SIZE {
            if i < buffer.len() {
                *ptr.add(i) = buffer[i];
            } else {
                *ptr.add(i) = 0;
            }
        }
    }
}
pub fn allocate_block() -> Option<usize> {
    unsafe {
        for i in 0..MAX_BLOCKS {
            let byte_idx = i / 8;
            let bit_idx = i % 8;
            if (BLOCK_BITMAP[byte_idx] & (1 << bit_idx)) == 0 {
                BLOCK_BITMAP[byte_idx] |= 1 << bit_idx;
                return Some(i);
            }
        }
    }
    None
}
