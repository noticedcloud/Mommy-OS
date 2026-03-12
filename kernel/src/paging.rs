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
        let (frame, _) = crate::pmm::allocate_frame_type(crate::pmm::MemoryType::Kernel).expect("OOM Paging");
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
        let (frame, _) = crate::pmm::allocate_frame_type(crate::pmm::MemoryType::Kernel).expect("OOM Paging");
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
        let (frame, _) = crate::pmm::allocate_frame_type(crate::pmm::MemoryType::Kernel).expect("OOM Paging");
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

pub unsafe fn create_process_pml4() -> u64 {
    let (pml4_frame, _) = crate::pmm::allocate_frame_type(crate::pmm::MemoryType::Kernel).expect("OOM PML4");
    if pml4_frame == 0 { panic!("OOM PML4"); }
    let pml4 = &mut *(pml4_frame as *mut PageTable);
    pml4.zero();

    let kernel_pml4 = active_level_4_table();

    pml4.entries[0] = kernel_pml4.entries[0];

    let (pdpt_frame, _) = crate::pmm::allocate_frame_type(crate::pmm::MemoryType::Kernel).unwrap();
    let pdpt = &mut *(pdpt_frame as *mut PageTable);
    pdpt.zero();
    pml4.entries[0] = pdpt_frame | PAGE_PRESENT | PAGE_WRITE | PAGE_USER;

    let kernel_pdpt_phys = kernel_pml4.entries[0] & 0x000FFFFFFFFFF000;
    let kernel_pdpt = &mut *(kernel_pdpt_phys as *mut PageTable);

    for i in 0..512 {
        pdpt.entries[i] = kernel_pdpt.entries[i];
    }

    let (pd_frame, _) = crate::pmm::allocate_frame_type(crate::pmm::MemoryType::Kernel).unwrap();
    let pd = &mut *(pd_frame as *mut PageTable);
    pd.zero();
    pdpt.entries[0] = pd_frame | PAGE_PRESENT | PAGE_WRITE | PAGE_USER;

    let kernel_pd_phys = kernel_pdpt.entries[0] & 0x000FFFFFFFFFF000;
    let kernel_pd = &mut *(kernel_pd_phys as *mut PageTable);

    for i in 0..512 {
        pd.entries[i] = kernel_pd.entries[i];
    }

    pml4_frame
}

pub unsafe fn map_user_page(pml4_phys: u64, phys_addr: u64, virt_addr: u64, flags: u64) {
    let pml4 = &mut *(pml4_phys as *mut PageTable);
    let p4_idx = (virt_addr >> 39) & 0x1FF;
    let p3_idx = (virt_addr >> 30) & 0x1FF;
    let p2_idx = (virt_addr >> 21) & 0x1FF;
    let p1_idx = (virt_addr >> 12) & 0x1FF;

    if pml4.entries[p4_idx as usize] & PAGE_PRESENT == 0 {
        let (frame, _) = crate::pmm::allocate_frame_type(crate::pmm::MemoryType::Kernel).unwrap();
        let table = &mut *(frame as *mut PageTable);
        table.zero();
        pml4.entries[p4_idx as usize] = frame | PAGE_PRESENT | PAGE_WRITE | PAGE_USER;
    }

    let p3_phys = pml4.entries[p4_idx as usize] & 0x000FFFFFFFFFF000;
    let p3 = &mut *(p3_phys as *mut PageTable);

    if p3.entries[p3_idx as usize] & PAGE_PRESENT == 0 {
        let (frame, _) = crate::pmm::allocate_frame_type(crate::pmm::MemoryType::Kernel).unwrap();
        let table = &mut *(frame as *mut PageTable);
        table.zero();
        p3.entries[p3_idx as usize] = frame | PAGE_PRESENT | PAGE_WRITE | PAGE_USER;
    }

    let p2_phys = p3.entries[p3_idx as usize] & 0x000FFFFFFFFFF000;
    let p2 = &mut *(p2_phys as *mut PageTable);

    if p2.entries[p2_idx as usize] & PAGE_PRESENT != 0 && p2.entries[p2_idx as usize] & PAGE_HUGE != 0 {
        let huge_phys_base = p2.entries[p2_idx as usize] & 0x000FFFFFFFE00000;
        let huge_flags = p2.entries[p2_idx as usize] & 0xFFF & !PAGE_HUGE;

        let (pt_frame, _) = crate::pmm::allocate_frame_type(crate::pmm::MemoryType::Kernel).unwrap();
        let pt = &mut *(pt_frame as *mut PageTable);
        pt.zero();

        for i in 0..512 {
            pt.entries[i] = (huge_phys_base + (i as u64 * 4096)) | huge_flags | PAGE_PRESENT;
        }

        p2.entries[p2_idx as usize] = pt_frame | PAGE_PRESENT | PAGE_WRITE | PAGE_USER;
    } else if p2.entries[p2_idx as usize] & PAGE_PRESENT == 0 {
        let (frame, _) = crate::pmm::allocate_frame_type(crate::pmm::MemoryType::Kernel).unwrap();
        let table = &mut *(frame as *mut PageTable);
        table.zero();
        p2.entries[p2_idx as usize] = frame | PAGE_PRESENT | PAGE_WRITE | PAGE_USER;
    }

    let p1_phys = p2.entries[p2_idx as usize] & 0x000FFFFFFFFFF000;
    let p1 = &mut *(p1_phys as *mut PageTable);

    p1.entries[p1_idx as usize] = phys_addr | flags | PAGE_PRESENT;
}

pub unsafe fn free_process_pml4(pml4_phys: u64) {
    let pml4 = &mut *(pml4_phys as *mut PageTable);

    if pml4.entries[0] & PAGE_PRESENT != 0 {
        let pdpt_phys = pml4.entries[0] & 0x000FFFFFFFFFF000;
        let pdpt = &mut *(pdpt_phys as *mut PageTable);

        if pdpt.entries[0] & PAGE_PRESENT != 0 {
            let pd_phys = pdpt.entries[0] & 0x000FFFFFFFFFF000;
            let pd = &mut *(pd_phys as *mut PageTable);

            for k in 0..512 {
                if pd.entries[k] & PAGE_PRESENT != 0 && pd.entries[k] & PAGE_HUGE == 0 {
                    let pt_phys = pd.entries[k] & 0x000FFFFFFFFFF000;
                    crate::pmm::free_frame(pt_phys);
                }
            }
            crate::pmm::free_frame(pd_phys);
        }
        crate::pmm::free_frame(pdpt_phys);
    }

    for i in 1..512 {
        if pml4.entries[i] & PAGE_PRESENT != 0 {
            let pdpt_phys = pml4.entries[i] & 0x000FFFFFFFFFF000;
            let pdpt = &mut *(pdpt_phys as *mut PageTable);
            for j in 0..512 {
                if pdpt.entries[j] & PAGE_PRESENT != 0 {
                    let pd_phys = pdpt.entries[j] & 0x000FFFFFFFFFF000;
                    let pd = &mut *(pd_phys as *mut PageTable);
                    for k in 0..512 {
                        if pd.entries[k] & PAGE_PRESENT != 0 && pd.entries[k] & PAGE_HUGE == 0 {
                            let pt_phys = pd.entries[k] & 0x000FFFFFFFFFF000;
                            crate::pmm::free_frame(pt_phys);
                        }
                    }
                    crate::pmm::free_frame(pd_phys);
                }
            }
            crate::pmm::free_frame(pdpt_phys);
        }
    }
    crate::pmm::free_frame(pml4_phys);
}
