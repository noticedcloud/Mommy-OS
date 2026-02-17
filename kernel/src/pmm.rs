const MAX_FRAMES: usize = 32768;
const BITMAP_SIZE: usize = MAX_FRAMES / 64;
static mut PMM_BITMAP: [u64; BITMAP_SIZE] = [0; BITMAP_SIZE];
static mut PMM_START: u64 = 0xA00000;
pub fn init() {
    unsafe {
        for i in 0..BITMAP_SIZE {
            PMM_BITMAP[i] = 0;
        }
        crate::serial::print_serial(b"[PMM] Bitmap Initialized (128MB)\n");
    }
}
pub fn allocate_frame() -> u64 {
    unsafe {
        for i in 0..BITMAP_SIZE {
            if PMM_BITMAP[i] != !0 {
                for j in 0..64 {
                    if (PMM_BITMAP[i] & (1 << j)) == 0 {
                        PMM_BITMAP[i] |= 1 << j;
                        let frame_idx = (i * 64) + j;
                        let addr = PMM_START + (frame_idx as u64 * 4096);
                        let ptr = addr as *mut u64;
                        for k in 0..512 {
                            *ptr.add(k) = 0;
                        }
                        return addr;
                    }
                }
            }
        }
        crate::serial::print_serial(b"[PMM] Out of Memory!\n");
        0
    }
}
pub fn free_frame(addr: u64) {
    if addr < unsafe { PMM_START } {
        return;
    }
    let frame_idx = (addr - unsafe { PMM_START }) / 4096;
    let bitmap_idx = (frame_idx / 64) as usize;
    let bit_idx = (frame_idx % 64) as usize;
    if bitmap_idx < BITMAP_SIZE {
        unsafe {
            PMM_BITMAP[bitmap_idx] &= !(1 << bit_idx);
        }
    }
}
