use core::arch::global_asm;

global_asm!(
    ".global switch_context",
    "switch_context:",
    "push rbx",
    "push rbp",
    "push r12",
    "push r13",
    "push r14",
    "push r15",
    "mov [rdi], rsp",
    "mov rsp, rsi",

    "mov rax, cr3",
    "cmp rax, rdx",
    "je .no_cr3_switch",
    "mov cr3, rdx",
    ".no_cr3_switch:",
    "pop r15",
    "pop r14",
    "pop r13",
    "pop r12",
    "pop rbp",
    "pop rbx",
    "ret",

    ".global process_bootstrap",
    ".extern exit_current",
    "process_bootstrap:",

    "iretq",
    "call exit_current",
    "hlt"
);

extern "C" {

    fn switch_context(old_rsp: *mut u64, new_rsp: u64, new_cr3: u64);

    fn process_bootstrap();
}

#[derive(Copy, Clone, PartialEq)]

pub enum AbiType {
    Native,
    Linux,
}

#[derive(Copy, Clone)]

pub struct Task {
    pub id: usize,
    pub rsp: u64,
    pub cr3: u64,
    pub kernel_stack: u64,
    pub active: bool,
    pub abi: AbiType,
    pub owned_frames: [u64; 16],
    pub binary_base: u64,
    pub binary_pages: usize,
}
const MAX_TASKS: usize = 16;
static mut TASKS: [Task; MAX_TASKS] = [Task {
    id: 0,
    rsp: 0,
    cr3: 0,
    kernel_stack: 0,
    active: false,
    abi: AbiType::Native,
    owned_frames: [0; 16],
    binary_base: 0,
    binary_pages: 0,
}; MAX_TASKS];
static mut CURRENT_TASK: usize = 0;
static mut NEXT_ID: usize = 1;
pub static mut FOREGROUND_TASK_ID: Option<usize> = None;

pub fn init() {
    unsafe {
        let mut cr3: u64;
        core::arch::asm!("mov {}, cr3", out(reg) cr3);
        TASKS[0].id = 0;
        TASKS[0].active = true;
        TASKS[0].rsp = 0;
        TASKS[0].cr3 = cr3;
        TASKS[0].kernel_stack = 0;
        TASKS[0].abi = AbiType::Native;
        TASKS[0].owned_frames = [0; 16];
        TASKS[0].binary_base = 0;
        TASKS[0].binary_pages = 0;
        CURRENT_TASK = 0;
        FOREGROUND_TASK_ID = None;
    }
}
pub fn spawn(f: extern "C" fn()) {
    unsafe {
        let mut idx = 0;
        let mut found = false;
        for i in 1..MAX_TASKS {
            if !TASKS[i].active {
                idx = i;
                found = true;
                break;
            }
        }
        if !found {
            return;
        }
        let stack_base = crate::pmm::allocate_contiguous_frames(4).expect("OOM Task Stack");
        let stack_ptr = stack_base + 16384;
        let sp = stack_ptr as *mut u64;
        let mut ptr = sp;
        ptr = ptr.sub(1); *ptr = 0;
        ptr = ptr.sub(1); *ptr = f as u64;
        ptr = ptr.sub(1); *ptr = 0;
        ptr = ptr.sub(1); *ptr = 0;
        ptr = ptr.sub(1); *ptr = 0;
        ptr = ptr.sub(1); *ptr = 0;
        ptr = ptr.sub(1); *ptr = 0;
        ptr = ptr.sub(1); *ptr = 0;

        let mut cr3: u64;
        core::arch::asm!("mov {}, cr3", out(reg) cr3);

        TASKS[idx].id = NEXT_ID;
        NEXT_ID += 1;
        TASKS[idx].rsp = ptr as u64;
        TASKS[idx].cr3 = cr3;
        TASKS[idx].kernel_stack = 0;
        TASKS[idx].active = true;
        TASKS[idx].abi = AbiType::Native;
        TASKS[idx].owned_frames = [0; 16];
        TASKS[idx].owned_frames[0] = stack_base;
    }
}
#[repr(C, packed)]

struct MomHeader {
    magic: [u8; 4],
    version: u8,
    type_flag: u8,
    compression: u8,
    padding: u8,
    capabilities: u64,
    entry_offset: u64,
    total_size: u64,
    stack_request: u64,
    checksum: u64,
    reserved: [u8; 16],
}
pub fn spawn_process(load_addr: u64, args: &[u8], abi: AbiType, binary_pages: usize) -> Option<usize> {
    unsafe {
        if abi == AbiType::Native {
            let header = &*(load_addr as *const MomHeader);
            if header.magic != [0x4D, 0x4F, 0x4D, 0x21] {
                crate::vga::print(b"Invalid MOM exe!\n");
                return None;
            }
        }

        let virt_base = if abi == AbiType::Native { 0x400000 } else { load_addr };
        let entry_point = if abi == AbiType::Native {
            let header = &*(load_addr as *const MomHeader);
            virt_base + header.entry_offset
        } else {
            virt_base
        };

        let mut idx = 0;
        let mut found = false;
        for i in 1..MAX_TASKS {
            if !TASKS[i].active {
                idx = i;
                found = true;
                break;
            }
        }
        if !found { return None; }

        let pml4_phys = crate::paging::create_process_pml4();

        for i in 0..binary_pages {
             crate::paging::map_user_page(
                 pml4_phys,
                 load_addr + (i as u64 * 4096),
                 virt_base + (i as u64 * 4096),
                 crate::paging::PAGE_WRITE | crate::paging::PAGE_USER
             );
        }

        let user_stack_base = crate::pmm::allocate_contiguous_frames(4).expect("OOM User Stack");
        let user_stack_top_virt = 0x00007FFFFFFF0000;
        let user_stack_base_virt = user_stack_top_virt - 16384;

        for i in 0..4 {
            crate::paging::map_user_page(
                pml4_phys,
                user_stack_base + (i as u64 * 4096),
                user_stack_base_virt + (i as u64 * 4096),
                crate::paging::PAGE_WRITE | crate::paging::PAGE_USER
            );
        }

        let user_stack_phys_top = user_stack_base + 16384;
        let mut sp_phys = user_stack_phys_top as *mut u8;
        sp_phys = sp_phys.sub(args.len() + 1);
        let sp_phys_aligned = (sp_phys as u64) & !0xF;
        sp_phys = sp_phys_aligned as *mut u8;
        for i in 0..args.len() {
            *sp_phys.add(i) = args[i];
        }
        *sp_phys.add(args.len()) = 0;

        let user_sp_virt = user_stack_top_virt - (user_stack_phys_top - sp_phys_aligned);

        let kernel_stack_base = crate::pmm::allocate_contiguous_frames(4).expect("OOM Kernel Stack");
        let kernel_stack_top = kernel_stack_base + 16384;

        let ptr_base = kernel_stack_base as *mut u64;
        for i in 0..(16384 / 8) { *ptr_base.add(i) = 0; }

        let mut ptr = kernel_stack_top as *mut u64;

        ptr = ptr.sub(1); *ptr = 0x23;
        ptr = ptr.sub(1); *ptr = user_sp_virt;
        ptr = ptr.sub(1); *ptr = 0x202;
        ptr = ptr.sub(1); *ptr = 0x2B;
        ptr = ptr.sub(1); *ptr = entry_point;

        ptr = ptr.sub(1); *ptr = process_bootstrap as *const () as u64;
        ptr = ptr.sub(1); *ptr = 0;
        ptr = ptr.sub(1); *ptr = 0;
        ptr = ptr.sub(1); *ptr = 0;
        ptr = ptr.sub(1); *ptr = 0;
        ptr = ptr.sub(1); *ptr = 0;
        ptr = ptr.sub(1); *ptr = 0;

        TASKS[idx].id = NEXT_ID;
        NEXT_ID += 1;
        TASKS[idx].rsp = ptr as u64;
        TASKS[idx].cr3 = pml4_phys;
        TASKS[idx].kernel_stack = kernel_stack_top;
        TASKS[idx].active = true;
        TASKS[idx].abi = abi;
        TASKS[idx].owned_frames = [0; 16];
        TASKS[idx].owned_frames[0] = user_stack_base;
        TASKS[idx].owned_frames[1] = kernel_stack_base;
        TASKS[idx].owned_frames[2] = pml4_phys;
        TASKS[idx].binary_base = load_addr;
        TASKS[idx].binary_pages = binary_pages;

        FOREGROUND_TASK_ID = Some(idx);
        let current = CURRENT_TASK;
        let next = idx;
        let old_rsp_ptr = &mut TASKS[current].rsp as *mut u64;
        let new_rsp = TASKS[next].rsp;

        if TASKS[next].kernel_stack != 0 {
            crate::gdt::TSS.rsp0 = TASKS[next].kernel_stack;
            crate::interrupts::CPU_LOCAL.kernel_stack = TASKS[next].kernel_stack;
        }

        CURRENT_TASK = next;
        switch_context(old_rsp_ptr, new_rsp, TASKS[next].cr3);

        Some(idx)
    }
}
#[no_mangle]
pub extern "C" fn exit_current() {
    unsafe {
        crate::serial::print_serial(b"[TASK] Exit Current\n");
        TASKS[CURRENT_TASK].active = false;

        let stack1 = TASKS[CURRENT_TASK].owned_frames[0];
        if stack1 != 0 {
            for i in 0..4 { crate::pmm::free_frame(stack1 + i * 4096); }
            TASKS[CURRENT_TASK].owned_frames[0] = 0;
        }

        let stack2 = TASKS[CURRENT_TASK].owned_frames[1];
        if stack2 != 0 {
            for i in 0..4 { crate::pmm::free_frame(stack2 + i * 4096); }
            TASKS[CURRENT_TASK].owned_frames[1] = 0;
        }

        let pml4 = TASKS[CURRENT_TASK].owned_frames[2];
        if pml4 != 0 {
            crate::paging::free_process_pml4(pml4);
            TASKS[CURRENT_TASK].owned_frames[2] = 0;
        }

        for i in 3..16 {
            TASKS[CURRENT_TASK].owned_frames[i] = 0;
        }

        let b_base = TASKS[CURRENT_TASK].binary_base;
        let b_pages = TASKS[CURRENT_TASK].binary_pages;
        if b_base != 0 && b_pages > 0 {
             for i in 0..b_pages {
                 crate::pmm::free_frame(b_base + (i as u64 * 4096));
             }
             TASKS[CURRENT_TASK].binary_base = 0;
             TASKS[CURRENT_TASK].binary_pages = 0;
        }
        if let Some(fg_idx) = FOREGROUND_TASK_ID {
            if CURRENT_TASK == fg_idx {
                FOREGROUND_TASK_ID = None;
                crate::shell::on_process_exit();
            }
        }
        schedule();
    }
}
pub fn schedule() {
    unsafe {
        let current = CURRENT_TASK;
        let mut next = current + 1;
        loop {
            if next >= MAX_TASKS {
                next = 0;
            }
            if TASKS[next].active {
                break;
            }
            next += 1;
        }
        if current != next {
            let old_rsp_ptr = &mut TASKS[current].rsp as *mut u64;
            let new_rsp = TASKS[next].rsp;

            if TASKS[next].kernel_stack != 0 {
                crate::gdt::TSS.rsp0 = TASKS[next].kernel_stack;
                crate::interrupts::CPU_LOCAL.kernel_stack = TASKS[next].kernel_stack;
            }

            CURRENT_TASK = next;
            switch_context(old_rsp_ptr, new_rsp, TASKS[next].cr3);
        }
    }
}
pub fn get_current_abi() -> AbiType {
    unsafe { TASKS[CURRENT_TASK].abi }
}

pub fn get_active_task_count() -> usize {
    let mut count = 0;
    unsafe {
        for i in 0..MAX_TASKS {
            if TASKS[i].active {
                count += 1;
            }
        }
    }
    count
}

pub fn get_current_pid() -> usize {
    unsafe { TASKS[CURRENT_TASK].id }
}

pub fn get_kernel_cr3() -> u64 {
    unsafe { TASKS[0].cr3 }
}
