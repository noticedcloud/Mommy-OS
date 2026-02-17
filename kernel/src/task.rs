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
    "mov rdi, r12",
    "mov rsi, r13",
    "sti",
    "call r14",
    "call exit_current",
    "hlt"
);
extern "C" {
    fn switch_context(old_rsp: *mut u64, new_rsp: u64);
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
    pub active: bool,
    pub abi: AbiType,
    pub owned_frames: [u64; 16],
}
const MAX_TASKS: usize = 16;
static mut TASKS: [Task; MAX_TASKS] = [Task {
    id: 0,
    rsp: 0,
    active: false,
    abi: AbiType::Native,
    owned_frames: [0; 16],
}; MAX_TASKS];
static mut CURRENT_TASK: usize = 0;
static mut NEXT_ID: usize = 1;
pub static mut FOREGROUND_TASK_ID: Option<usize> = None;
pub fn init() {
    unsafe {
        TASKS[0].id = 0;
        TASKS[0].active = true;
        TASKS[0].rsp = 0;
        TASKS[0].abi = AbiType::Native;
        TASKS[0].owned_frames = [0; 16];
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
        let stack_ptr = crate::pmm::allocate_frame() + 4096;
        let sp = stack_ptr as *mut u64;
        let mut ptr = sp;
        unsafe {
            crate::serial::print_serial(b"Spawn F: ");
            crate::serial::print_hex(f as u64);
            crate::serial::print_serial(b" SP: ");
            crate::serial::print_hex(sp as u64);
            crate::serial::print_serial(b" SwitchCtx: ");
            crate::serial::print_hex(switch_context as usize as u64);
            crate::serial::print_serial(b"\n");
        }
        ptr = ptr.sub(1);
        *ptr = 0;
        ptr = ptr.sub(1);
        *ptr = f as u64;
        ptr = ptr.sub(1);
        *ptr = 0;
        ptr = ptr.sub(1);
        *ptr = 0;
        ptr = ptr.sub(1);
        *ptr = 0;
        ptr = ptr.sub(1);
        *ptr = 0;
        ptr = ptr.sub(1);
        *ptr = 0;
        ptr = ptr.sub(1);
        *ptr = 0;
        TASKS[idx].id = NEXT_ID;
        NEXT_ID += 1;
        TASKS[idx].rsp = ptr as u64;
        TASKS[idx].active = true;
        TASKS[idx].abi = AbiType::Native;
        TASKS[idx].owned_frames = [0; 16];
        TASKS[idx].owned_frames[0] = stack_ptr - 4096;
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
pub fn spawn_process(load_addr: u64, args: &[u8], abi: AbiType) -> Option<usize> {
    unsafe {
        if abi == AbiType::Native {
            let header = &*(load_addr as *const MomHeader);
            if header.magic != [0x4D, 0x4F, 0x4D, 0x21] {
                crate::vga::print(b"Invalid MOM exe!\n");
                return None;
            }
        }
        let entry_point = if abi == AbiType::Native {
            let header = &*(load_addr as *const MomHeader);
            load_addr + header.entry_offset
        } else {
            load_addr
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
        if !found {
            return None;
        }
        let stack_base = crate::pmm::allocate_frame();
        unsafe {
            crate::serial::print_serial(b"[TASK] Stack Base: ");
            crate::serial::print_hex(stack_base);
            crate::serial::print_serial(b"\n");
        }
        let stack_top = stack_base + 4096;
        let ptr_base = stack_base as *mut u64;
        for i in 0..(4096 / 8) {
            *ptr_base.add(i) = 0;
        }
        let mut sp = stack_top as *mut u8;
        sp = sp.sub(args.len() + 1);
        let sp_addr_args = sp as u64;
        let sp_aligned_args = sp_addr_args & !0xF;
        sp = sp_aligned_args as *mut u8;
        let args_ptr = sp;
        for i in 0..args.len() {
            *sp.add(i) = args[i];
        }
        *sp.add(args.len()) = 0;
        let sp_addr = sp as u64;
        let sp_aligned = sp_addr & !0xF;
        let mut ptr = sp_aligned as *mut u64;
        ptr = ptr.sub(1);
        *ptr = process_bootstrap as u64;
        ptr = ptr.sub(1);
        *ptr = 0;
        ptr = ptr.sub(1);
        *ptr = 0;
        ptr = ptr.sub(1);
        *ptr = args.len() as u64;
        ptr = ptr.sub(1);
        *ptr = sp as u64;
        ptr = ptr.sub(1);
        *ptr = entry_point;
        ptr = ptr.sub(1);
        *ptr = 0;
        TASKS[idx].id = NEXT_ID;
        NEXT_ID += 1;
        TASKS[idx].rsp = ptr as u64;
        TASKS[idx].active = true;
        TASKS[idx].abi = abi;
        TASKS[idx].owned_frames = [0; 16];
        TASKS[idx].owned_frames[0] = stack_base;
        FOREGROUND_TASK_ID = Some(idx);
        unsafe {
            let current = CURRENT_TASK;
            let next = idx;
            let old_rsp_ptr = &mut TASKS[current].rsp as *mut u64;
            let new_rsp = TASKS[next].rsp;
            CURRENT_TASK = next;
            crate::task::switch_context(old_rsp_ptr, new_rsp);
        }
        Some(idx)
    }
}
#[no_mangle]
pub extern "C" fn exit_current() {
    unsafe {
        crate::serial::print_serial(b"[TASK] Exit Current\n");
        TASKS[CURRENT_TASK].active = false;
        for i in 0..16 {
            let frame = TASKS[CURRENT_TASK].owned_frames[i];
            if frame != 0 {
                crate::pmm::free_frame(frame);
                TASKS[CURRENT_TASK].owned_frames[i] = 0;
            }
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
            CURRENT_TASK = next;
            switch_context(old_rsp_ptr, new_rsp);
        }
    }
}
pub fn get_current_abi() -> AbiType {
    unsafe { TASKS[CURRENT_TASK].abi }
}
