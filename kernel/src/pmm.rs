const MAX_FRAMES: usize = 16384;
const BITMAP_SIZE: usize = MAX_FRAMES / 64;
static mut PMM_BITMAP: [u64; BITMAP_SIZE] = [0; BITMAP_SIZE];
static mut PMM_INVASION_BITMAP: [u64; BITMAP_SIZE] = [0; BITMAP_SIZE];
static mut PMM_START: u64 = 0xA00000;
static mut SPLIT_INDEX: usize = MAX_FRAMES / 2;
static mut KERNEL_USED: usize = 0;
static mut CRADLE_USED: usize = 0;
static mut PLAYPEN_USED: usize = 0;
static mut PLAYPEN_INVASION_COUNT: usize = 0;

pub static mut TOTAL_KERNEL_ALLOCS: usize = 0;
pub static mut TOTAL_CRADLE_ALLOCS: usize = 0;
pub static mut TOTAL_PLAYPEN_ALLOCS: usize = 0;
pub static mut TOTAL_INVASIONS: usize = 0;

#[derive(PartialEq, Copy, Clone)]

pub enum MemoryType {
    Kernel,
    Cradle,
    Playpen,
}

#[derive(Copy, Clone)]

struct Frame {
    owner_pid: usize,
    memory_type: MemoryType,
    encrypted: bool,
}

static mut FRAME_METADATA: [Frame; MAX_FRAMES] = [Frame {
    owner_pid: 0,
    memory_type: MemoryType::Kernel,
    encrypted: false,
}; MAX_FRAMES];

#[derive(Clone, Copy)]
#[repr(C, packed)]

struct E820Entry {
    base_addr: u64,
    length: u64,
    region_type: u32,
    acpi_attr: u32,
}

pub fn init() {
    unsafe {

        for i in 0..BITMAP_SIZE {
            PMM_BITMAP[i] = !0;
            PMM_INVASION_BITMAP[i] = 0;
        }
        for i in 0..MAX_FRAMES {
            FRAME_METADATA[i] = Frame {
                owner_pid: 0,
                memory_type: MemoryType::Kernel,
                encrypted: false,
            };
        }

        let e820_count = *(0x0500 as *const u32);
        if e820_count > 0 && e820_count < 100 {
            let e820_entries = core::slice::from_raw_parts(0x0504 as *const E820Entry, e820_count as usize);

            for entry in e820_entries {
                if entry.region_type == 1 {
                    let start = entry.base_addr;
                    let end = start + entry.length;

                    let pmm_start = PMM_START as u64;
                    let pmm_end = pmm_start + (MAX_FRAMES as u64 * 4096);

                    let overlap_start = core::cmp::max(start, pmm_start);
                    let overlap_end = core::cmp::min(end, pmm_end);

                    if overlap_start < overlap_end {
                        let start_frame = ((overlap_start - pmm_start + 4095) / 4096) as usize;
                        let end_frame = ((overlap_end - pmm_start) / 4096) as usize;

                        for i in start_frame..end_frame {
                            let bitmap_idx = i / 64;
                            let bit_idx = i % 64;
                            PMM_BITMAP[bitmap_idx] &= !(1 << bit_idx);
                        }
                    }
                }
            }
        } else {

            for i in 0..BITMAP_SIZE {
                PMM_BITMAP[i] = 0;
            }
        }

        SPLIT_INDEX = MAX_FRAMES / 2;
        crate::serial::print_serial(b"[PMM] Cradle & Playpen Initialized. Map size: ");
        crate::serial::print_u64(e820_count as u64);
        crate::serial::print_serial(b"\n");
    }
}

fn get_frame_state(idx: usize) -> bool {
    unsafe {
        let bitmap_idx = idx / 64;
        let bit_idx = idx % 64;
        (PMM_BITMAP[bitmap_idx] & (1 << bit_idx)) != 0
    }
}

fn set_frame_state(idx: usize, used: bool) {
    unsafe {
        let bitmap_idx = idx / 64;
        let bit_idx = idx % 64;
        if used {
            PMM_BITMAP[bitmap_idx] |= 1 << bit_idx;
        } else {
            PMM_BITMAP[bitmap_idx] &= !(1 << bit_idx);
        }
    }
}

fn set_invasion_state(idx: usize, is_invasion: bool) {
    unsafe {
        let bitmap_idx = idx / 64;
        let bit_idx = idx % 64;
        if is_invasion {
            PMM_INVASION_BITMAP[bitmap_idx] |= 1 << bit_idx;
        } else {
            PMM_INVASION_BITMAP[bitmap_idx] &= !(1 << bit_idx);
        }
    }
}

fn is_invasion_frame(idx: usize) -> bool {
    unsafe {
        let bitmap_idx = idx / 64;
        let bit_idx = idx % 64;
        (PMM_INVASION_BITMAP[bitmap_idx] & (1 << bit_idx)) != 0
    }
}

unsafe fn zero_frame(addr: u64) {
    let ptr = addr as *mut u64;
    for k in 0..512 {
        *ptr.add(k) = 0;
    }
}

pub fn allocate_frame_type(mtype: MemoryType) -> Option<(u64, bool)> {
    unsafe {
        let split = SPLIT_INDEX;
        let total = MAX_FRAMES;

        match mtype {
            MemoryType::Kernel => {
                for i in split..total {
                    if !get_frame_state(i) {
                        set_frame_state(i, true);
                        KERNEL_USED += 1;
                        TOTAL_KERNEL_ALLOCS += 1;
                        FRAME_METADATA[i].memory_type = MemoryType::Kernel;
                        FRAME_METADATA[i].owner_pid = 0;
                        FRAME_METADATA[i].encrypted = false;
                        let addr = PMM_START + (i as u64 * 4096);
                        zero_frame(addr);
                        return Some((addr, true));
                    }
                }
            }
            MemoryType::Cradle => {
                for i in split..total {
                    if !get_frame_state(i) {
                        set_frame_state(i, true);
                        CRADLE_USED += 1;
                        TOTAL_CRADLE_ALLOCS += 1;
                        FRAME_METADATA[i].memory_type = MemoryType::Cradle;
                        FRAME_METADATA[i].owner_pid = crate::task::get_current_pid();
                        FRAME_METADATA[i].encrypted = false;
                        let addr = PMM_START + (i as u64 * 4096);
                        zero_frame(addr);
                        return Some((addr, true));
                    }
                }
                for i in 0..split {
                    if !get_frame_state(i) {
                        set_frame_state(i, true);
                        PLAYPEN_USED += 1;
                        TOTAL_PLAYPEN_ALLOCS += 1;
                        FRAME_METADATA[i].memory_type = MemoryType::Playpen;
                        FRAME_METADATA[i].owner_pid = crate::task::get_current_pid();
                        let addr = PMM_START + (i as u64 * 4096);
                        zero_frame(addr);
                        return Some((addr, false));
                    }
                }
            }
            MemoryType::Playpen => {
                for i in 0..split {
                    if !get_frame_state(i) {
                        set_frame_state(i, true);
                        PLAYPEN_USED += 1;
                        TOTAL_PLAYPEN_ALLOCS += 1;
                        FRAME_METADATA[i].memory_type = MemoryType::Playpen;
                        FRAME_METADATA[i].owner_pid = crate::task::get_current_pid();
                        let addr = PMM_START + (i as u64 * 4096);
                        zero_frame(addr);
                        return Some((addr, true));
                    }
                }

                let cradle_total = total - split;
                let cradle_free = cradle_total - CRADLE_USED - PLAYPEN_INVASION_COUNT;
                let free_percent = (cradle_free * 100) / cradle_total;

                if free_percent > 20 {
                    let invasion_percent = (PLAYPEN_INVASION_COUNT * 100) / cradle_total;
                    if invasion_percent < 10 {
                        for i in split..total {
                             if !get_frame_state(i) {
                                 set_frame_state(i, true);
                                 set_invasion_state(i, true);
                                 PLAYPEN_INVASION_COUNT += 1;
                                 TOTAL_INVASIONS += 1;
                                 FRAME_METADATA[i].memory_type = MemoryType::Playpen;
                                 FRAME_METADATA[i].owner_pid = crate::task::get_current_pid();
                                 let addr = PMM_START + (i as u64 * 4096);
                                 zero_frame(addr);
                                 return Some((addr, true));
                             }
                        }
                    }
                }
            }
        }
        None
    }
}

pub fn allocate_contiguous_frames(count: usize) -> Option<u64> {
    unsafe {
        let total = MAX_FRAMES;
        let split = SPLIT_INDEX;

        for i in 0..(split - count) {
            let mut found = true;
            for j in 0..count {
                if get_frame_state(i + j) {
                    found = false;
                    break;
                }
            }
            if found {
                for j in 0..count {
                    set_frame_state(i + j, true);
                    FRAME_METADATA[i + j].memory_type = MemoryType::Playpen;
                    FRAME_METADATA[i + j].owner_pid = crate::task::get_current_pid();
                    let addr = PMM_START + ((i + j) as u64 * 4096);
                    zero_frame(addr);
                }
                PLAYPEN_USED += count;
                TOTAL_PLAYPEN_ALLOCS += count;
                return Some(PMM_START + (i as u64 * 4096));
            }
        }

        for i in split..(total - count) {
            let mut found = true;
            for j in 0..count {
                if get_frame_state(i + j) {
                    found = false;
                    break;
                }
            }
            if found {
                for j in 0..count {
                    set_frame_state(i + j, true);
                    FRAME_METADATA[i + j].memory_type = MemoryType::Cradle;
                    FRAME_METADATA[i + j].owner_pid = crate::task::get_current_pid();
                    let addr = PMM_START + ((i + j) as u64 * 4096);
                    zero_frame(addr);
                }
                CRADLE_USED += count;
                TOTAL_CRADLE_ALLOCS += count;
                return Some(PMM_START + (i as u64 * 4096));
            }
        }
        None
    }
}

pub fn allocate_frame() -> u64 {
    if let Some((addr, _)) = allocate_frame_type(MemoryType::Playpen) {
        addr
    } else {
        crate::serial::print_serial(b"[PMM] OOM\n");
        0
    }
}

pub fn free_frame(addr: u64) {
    unsafe {
        if addr < PMM_START { return; }
        let frame_idx = ((addr - PMM_START) / 4096) as usize;
        if frame_idx >= MAX_FRAMES { return; }

        if get_frame_state(frame_idx) {
             set_frame_state(frame_idx, false);

             if frame_idx < SPLIT_INDEX {
                 if PLAYPEN_USED > 0 { PLAYPEN_USED -= 1; }
             } else {
                 if is_invasion_frame(frame_idx) {
                     set_invasion_state(frame_idx, false);
                     if PLAYPEN_INVASION_COUNT > 0 { PLAYPEN_INVASION_COUNT -= 1; }
                 } else {
                     let mtype = FRAME_METADATA[frame_idx].memory_type;
                     match mtype {
                         MemoryType::Kernel => {
                             if KERNEL_USED > 0 { KERNEL_USED -= 1; }
                         }
                         MemoryType::Cradle => {
                             if CRADLE_USED > 0 { CRADLE_USED -= 1; }
                         }
                         MemoryType::Playpen => {
                             if PLAYPEN_USED > 0 { PLAYPEN_USED -= 1; }
                         }
                     }
                 }
             }
        }
    }
}
pub fn get_stats() -> (usize, usize, usize, usize, usize, usize, usize, usize, usize) {
    unsafe {
        (KERNEL_USED, CRADLE_USED, PLAYPEN_USED, TOTAL_KERNEL_ALLOCS, TOTAL_CRADLE_ALLOCS, TOTAL_PLAYPEN_ALLOCS, TOTAL_INVASIONS, SPLIT_INDEX, MAX_FRAMES)
    }
}
