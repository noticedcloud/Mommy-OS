use crate::pmm::allocate_frame;
use core::arch::asm;
pub const PAGE_PRESENT: u64 = 1 << 0;
pub const PAGE_WRITE: u64 = 1 << 1;
pub const PAGE_USER: u64 = 1 << 2;
pub const PAGE_NO_CACHE: u64 = 1 << 4;
pub const PAGE_HUGE: u64 = 1 << 7;
#[repr(C, align(4096))]
pub struct PageTable {
    pub entries: [u64; 512],
}
impl PageTable {
    pub fn zero(&mut self) {
        for i in 0..512 {
            self.entries[i] = 0;
        }
    }
}
pub unsafe fn active_level_4_table() -> &'static mut PageTable {
    let mut cr3: u64;
    asm!("mov {}, cr3", out(reg) cr3);
    let phys = cr3 & 0x000FFFFFFFFFF000;
    &mut *(phys as *mut PageTable)
}
pub unsafe fn map_page(phys_addr: u64, virt_addr: u64, flags: u64) {
    let p4 = active_level_4_table();
    let p4_idx = (virt_addr >> 39) & 0x1FF;
    let p3_idx = (virt_addr >> 30) & 0x1FF;
    let p2_idx = (virt_addr >> 21) & 0x1FF;
    let p1_idx = (virt_addr >> 12) & 0x1FF;
    if p4.entries[p4_idx as usize] & PAGE_PRESENT == 0 {
        let frame = allocate_frame();
        if frame == 0 {
            panic!("OOM Paging P3");
        }
        let table = frame as *mut PageTable;
        (*table).zero();
        p4.entries[p4_idx as usize] = frame | PAGE_PRESENT | PAGE_WRITE | PAGE_USER;
    }
    let p3_phys = p4.entries[p4_idx as usize] & 0x000FFFFFFFFFF000;
    let p3 = &mut *(p3_phys as *mut PageTable);
    if p3.entries[p3_idx as usize] & PAGE_PRESENT == 0 {
        let frame = allocate_frame();
        if frame == 0 {
            panic!("OOM Paging P2");
        }
        let table = frame as *mut PageTable;
        (*table).zero();
        p3.entries[p3_idx as usize] = frame | PAGE_PRESENT | PAGE_WRITE | PAGE_USER;
    }
    let p2_phys = p3.entries[p3_idx as usize] & 0x000FFFFFFFFFF000;
    let p2 = &mut *(p2_phys as *mut PageTable);
    if p2.entries[p2_idx as usize] & PAGE_PRESENT == 0 {
        let frame = allocate_frame();
        if frame == 0 {
            panic!("OOM Paging P1");
        }
        let table = frame as *mut PageTable;
        (*table).zero();
        p2.entries[p2_idx as usize] = frame | PAGE_PRESENT | PAGE_WRITE | PAGE_USER;
    } else if (p2.entries[p2_idx as usize] & PAGE_HUGE) != 0 {
    }
    let p1_phys = p2.entries[p2_idx as usize] & 0x000FFFFFFFFFF000;
    let p1 = &mut *(p1_phys as *mut PageTable);
    if p1.entries[p1_idx as usize] & PAGE_PRESENT != 0 {}
    p1.entries[p1_idx as usize] = phys_addr | flags | PAGE_PRESENT;
    asm!("invlpg [{}]", in(reg) virt_addr);
}
pub unsafe fn identity_map(addr: u64, flags: u64) {
    map_page(addr, addr, flags);
}
