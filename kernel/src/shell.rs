use crate::fs::directory::DirEntry;
use crate::fs::inode::{get_inode, FileType};
use crate::fs::ramdisk::read_block;
use crate::fs::ROOT_INODE;
use crate::vga::{print, set_input_start};
static mut INPUT_BUFFER: [u8; 256] = [0; 256];
static mut BUFFER_IDX: usize = 0;
pub static mut CURRENT_DIR: usize = 0;
static mut KEY_BUFFER: [u8; 128] = [0; 128];
static mut KEY_HEAD: usize = 0;
static mut KEY_TAIL: usize = 0;

pub fn init_shell() {
    unsafe {
        CURRENT_DIR = ROOT_INODE;
        BUFFER_IDX = 0;
        for i in 0..256 {
            INPUT_BUFFER[i] = 0;
        }
    }
}

pub fn on_process_exit() {
    print(b"Mommy is listening, write darling <3 ");
    set_input_start();
}

pub fn handle_key(c: u8) {
    unsafe {
        if (KEY_TAIL + 1) % 128 != KEY_HEAD {
            KEY_BUFFER[KEY_TAIL] = c;
            KEY_TAIL = (KEY_TAIL + 1) % 128;
        }
    }
}

fn process_key(c: u8) {
    unsafe {
        if (*(&raw const crate::task::FOREGROUND_TASK_ID)).is_some() {
            return;
        }
        if c == b'\n' {
            print(&[b'\n']);
            let is_async = execute_command();
            BUFFER_IDX = 0;
            for i in 0..256 {
                INPUT_BUFFER[i] = 0;
            }
            if !is_async {
                print(b"Mommy is listening, write darling <3 ");
                set_input_start();
            }
        } else if c == 0x08 {
            if BUFFER_IDX > 0 {
                BUFFER_IDX -= 1;
                INPUT_BUFFER[BUFFER_IDX] = 0;
                print(&[0x08]);
            }
        } else if c == b'\r' {

        } else if c != 0 && BUFFER_IDX < 255 {
            INPUT_BUFFER[BUFFER_IDX] = c;
            BUFFER_IDX += 1;
            print(&[c]);
        }
    }
}

pub extern "C" fn shell_task() {
    print(b"[SHELL] Task Started.\n");
    print(b"Mommy is listening, write darling <3 ");
    set_input_start();

    loop {
        let key = crate::interrupts::without_interrupts(|| unsafe {
            if KEY_HEAD != KEY_TAIL {
                let c = KEY_BUFFER[KEY_HEAD];
                KEY_HEAD = (KEY_HEAD + 1) % 128;
                Some(c)
            } else {
                None
            }
        });

        if let Some(c) = key {
            process_key(c);
        } else {
            unsafe {
                core::arch::asm!("sti", "hlt", "cli");
            }
        }
    }
}
fn execute_command() -> bool {
    unsafe {
        let input = &INPUT_BUFFER[..BUFFER_IDX];
        let mut split_idx = 0;
        let mut found_space = false;
        for i in 0..input.len() {
            if input[i] == b' ' {
                split_idx = i;
                found_space = true;
                break;
            }
        }
        let mut cmd = if found_space {
            &input[..split_idx]
        } else {
            input
        };

        while cmd.len() > 0 && (cmd[0] == b' ' || cmd[0] == b'\t') { cmd = &cmd[1..]; }
        while cmd.len() > 0 && (cmd[cmd.len()-1] == b' ' || cmd[cmd.len()-1] == b'\t' || cmd[cmd.len()-1] == b'\r') { cmd = &cmd[..cmd.len()-1]; }

        if cmd == b"help" {
            print(b"Available commands:\n");
            print(b"  cd       - Change directory\n");
            print(b"  pwd      - specific current path\n");
            print(b"  ... and commands in /CHEST/WHIP:\n");
            print(b"  ls       - List files\n");
            print(b"  clear    - Clear screen\n");
            print(b"  echo     - Print text\n");
            print(b"  reboot   - Reboot system\n");
            print(b"  shutdown - Shutdown system\n");
        } else if cmd == b"debug" {
            crate::fs::verify_fs();
        } else if cmd == b"mommy" {
            print(b"Darling, hello, what do you need from mommy? <3\n");
        } else if cmd == b"pwd" {
            print_cwd(CURRENT_DIR);
            print(b"\n");
        } else if cmd == b"cd" {
            if found_space {
                change_dir(&input[split_idx + 1..]);
            } else {
                print(b"Dove vuoi andare, amore?\n");
            }
        } else if cmd == b"zoom" {
            if found_space {
                let mut arg = &input[split_idx + 1..];
                if arg.len() > 0 && (arg[arg.len() - 1] == b'x' || arg[arg.len() - 1] == b'X') {
                    arg = &arg[..arg.len() - 1];
                }
                let mut dot_pos = None;
                for i in 0..arg.len() {
                    if arg[i] == b'.' {
                        dot_pos = Some(i);
                        break;
                    }
                }
                let (num, den) = if let Some(dot) = dot_pos {
                    let int_part_str = &arg[..dot];
                    let frac_part_str = &arg[dot + 1..];
                    let mut int_val: usize = 0;
                    let mut frac_val: usize = 0;
                    let mut den: usize = 1;
                    for &b in int_part_str {
                        if b >= b'0' && b <= b'9' {
                            int_val = int_val * 10 + (b - b'0') as usize;
                        }
                    }
                    for &b in frac_part_str {
                        if b >= b'0' && b <= b'9' {
                            frac_val = frac_val * 10 + (b - b'0') as usize;
                            den *= 10;
                        }
                    }
                    (int_val * den + frac_val, den)
                } else {
                    let mut int_val: usize = 0;
                    for &b in arg {
                        if b >= b'0' && b <= b'9' {
                            int_val = int_val * 10 + (b - b'0') as usize;
                        }
                    }
                    (int_val, 1)
                };
                if num == 0 {
                    print(b"Zoom invalido (0)!\n");
                } else {
                    crate::vesa::set_zoom(num, den);
                    crate::vesa::clear_screen(0);
                    crate::vesa::CURSOR_X = 0;
                    crate::vesa::CURSOR_Y = 0;
                    print(b"Zoom impostato a: ");
                    print(arg);
                    print(b"x\n");
                }
            } else {
                print(b"Uso: zoom <valore> (es. 1.5x)\n");
            }
        } else if cmd == b"pci" || cmd == b"lspci" {
            crate::drivers::pci::lspci();
        } else if cmd == b"net_test" {
            let pkt = [
                0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x52, 0x54, 0x00, 0x12, 0x34, 0x56, 0x08, 0x06,
                0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0x4D, 0x6F, 0x6D, 0x6D, 0x79,
            ];
            crate::drivers::e1000::send_packet(&pkt);
        } else if cmd == b"ping" {
            let mut space_idx = 0;
            let mut has_arg = false;
            for i in 0..input.len() {
                if input[i] == b' ' {
                    space_idx = i;
                    has_arg = true;
                    break;
                }
            }
            if has_arg {
                let arg = &input[space_idx + 1..];
                let arg_str = core::str::from_utf8(arg).unwrap_or("");
                let trim_str = arg_str.trim();
                if trim_str.len() > 0 {
                    if let Some(ip) = parse_ip(trim_str) {
                        crate::net::ping_blocking(ip);
                    } else if let Some(ip) = crate::net::dns::resolve_hostname(trim_str) {
                        print(b"Resolved ");
                        print(trim_str.as_bytes());
                        print(b" to ");
                        crate::net::print_ip(ip);
                        print(b"\n");
                        crate::net::ping_blocking(ip);
                    } else {
                        print(b"Host not found.\n");
                    }
                }
            } else {
                crate::net::ping_blocking(crate::net::GATEWAY_IP);
            }
            return false;
        } else if cmd == b"resolution" {
            if found_space {
                print(b"Cambio risoluzione richiede modifica bootloader e riavvio.\n");
                print(b"Risoluzione attuale: 1920x1080\n");
            } else {
                print(b"Uso: resolution <WxH>\n");
            }
        } else if cmd.starts_with(b"./") {
            let filename = &cmd[2..];
            return execute_binary(filename, CURRENT_DIR);
        } else if cmd.ends_with(b".MOM") || cmd.ends_with(b".mom") {
            return execute_binary(cmd, CURRENT_DIR);
        } else if cmd.is_empty() {
        } else {
            if let Some(inode) = resolve_system_bin(cmd) {
                let args = if found_space {
                    &input[split_idx + 1..]
                } else {
                    b""
                };
                return execute_binary_inode(inode, args);
            }
            print(b"Mommy doesn't understand: ");
            print(cmd);
            print(b"\n");
        }
        false
    }
}
unsafe fn resolve_system_bin(name: &[u8]) -> Option<usize> {
    let mut filename_buf = [0u8; 64];
    if name.len() + 4 > 64 {
        return None;
    }
    for i in 0..name.len() {
        filename_buf[i] = name[i];
    }
    filename_buf[name.len()] = b'.';
    filename_buf[name.len() + 1] = b'm';
    filename_buf[name.len() + 2] = b'o';
    filename_buf[name.len() + 3] = b'm';
    let filename = &filename_buf[..name.len() + 4];
    let mut buffer = [0u8; 4096];
    let root = get_inode(ROOT_INODE).unwrap();
    if let Some(chest_idx) = crate::fs::directory::find_entry(root, b"CHEST", &mut buffer) {
        let chest = get_inode(chest_idx as usize).unwrap();
        if let Some(whip_idx) = crate::fs::directory::find_entry(chest, b"WHIP", &mut buffer) {
            let whip = get_inode(whip_idx as usize).unwrap();
            if let Some(bin_idx) = crate::fs::directory::find_entry(whip, filename, &mut buffer) {
                return Some(bin_idx as usize);
            }
        }
    }
    if let Some(lap_idx) = crate::fs::directory::find_entry(root, b"LAP", &mut buffer) {
        let lap = get_inode(lap_idx as usize).unwrap();
        if let Some(user_idx) = crate::fs::directory::find_entry(lap, b"darling", &mut buffer) {
            let user = get_inode(user_idx as usize).unwrap();
            if let Some(toys_idx) = crate::fs::directory::find_entry(user, b"TOYS", &mut buffer) {
                let toys = get_inode(toys_idx as usize).unwrap();
                if let Some(bin_idx) = crate::fs::directory::find_entry(toys, filename, &mut buffer) {
                        crate::serial::print_serial(b"[SHELL] Resolved System Bin: ");
                        crate::serial::print_serial(filename);
                        crate::serial::print_serial(b" -> Inode ");
                        crate::serial::print_hex(bin_idx as u64);
                        crate::serial::print_serial(b"\n");
                    return Some(bin_idx as usize);
                }
            }
        }
    }
    None
}
unsafe fn execute_binary(name: &[u8], dir_inode: usize) -> bool {
    let current_inode = get_inode(dir_inode).unwrap();
    let mut buffer = [0u8; 4096];
    if let Some(inode_idx) = find_entry_in_dir(current_inode, name, &mut buffer) {
            crate::serial::print_serial(b"[SHELL] Found Inode: ");
            crate::serial::print_hex(inode_idx as u64);
            crate::serial::print_serial(b"\n");
        return execute_binary_inode(inode_idx as usize, b"");
    } else {
        print(b"File not found, darling!\n");
    }
    false
}
unsafe fn execute_binary_inode(inode_idx: usize, args: &[u8]) -> bool {
    crate::serial::print_serial(b"[SHELL] Executing Inode: ");
    crate::serial::print_hex(inode_idx as u64);
    crate::serial::print_serial(b"\n");
    let inode = get_inode(inode_idx).unwrap();
    if inode.file_type == FileType::File {
        let mut buffer = [0u8; 4096];
        crate::fs::ramdisk::read_block(inode.block as usize, &mut buffer);
        crate::serial::print_serial(b"[SHELL] Read header: ");
        for k in 0..4 {
            crate::serial::print_hex(buffer[k] as u64);
            crate::serial::print_serial(b" ");
        }
        crate::serial::print_serial(b"\n");
        if buffer[0] == 0x7F && buffer[1] == b'E' && buffer[2] == b'L' && buffer[3] == b'F' {
            crate::serial::print_serial(b"[SHELL] Detected ELF Binary!\n");
            let elf_hdr = &*(buffer.as_ptr() as *const crate::elf::Elf64hdr);
            if elf_hdr.e_machine != crate::elf::EM_X86_64 {
                print(b"Not a valid x86_64 binary!\n");
                return false;
            }
            let ph_off = elf_hdr.e_phoff;
            let ph_num = elf_hdr.e_phnum;
            let ph_ent_size = elf_hdr.e_phentsize;
            if ph_off + (ph_num as u64 * ph_ent_size as u64) > 4096 {
                print(b"ELF Header too big for my tummy!\n");
                return false;
            }
            let ph_base = buffer.as_ptr().add(ph_off as usize);
            let mut elf_load_base: u64 = u64::MAX;
            let mut elf_load_end: u64 = 0;
            for i in 0..ph_num {
                let ph = &*(ph_base.add(i as usize * ph_ent_size as usize)
                    as *const crate::elf::Elf64Phdr);
                if ph.p_type == crate::elf::PT_LOAD {
                    if ph.p_vaddr < elf_load_base {
                        elf_load_base = ph.p_vaddr;
                    }
                    let seg_end = ph.p_vaddr + ph.p_memsz;
                    if seg_end > elf_load_end {
                        elf_load_end = seg_end;
                    }
                    let dest = ph.p_vaddr as *mut u8;
                    crate::serial::print_serial(b"[SHELL] Loading Segment to ");
                    crate::serial::print_hex(ph.p_vaddr);
                    crate::serial::print_serial(b"\n");
                    let src_offset = ph.p_offset;
                    let len = ph.p_filesz;
                    let _start_block = (src_offset / 4096) as usize;
                    let _end_block = ((src_offset + len + 4095) / 4096) as usize;
                    let mut copied = 0;
                    let mut _current_offset = src_offset;
                    let mut current_blk_idx = inode.block as usize + (src_offset / 4096) as usize;
                    let mut block_offset = (src_offset % 4096) as usize;
                    while copied < len {
                        crate::fs::ramdisk::read_block(current_blk_idx, &mut buffer);
                        let chunk = if block_offset + (len - copied) as usize > 4096 {
                            4096 - block_offset
                        } else {
                            (len - copied) as usize
                        };
                        for k in 0..chunk {
                            *dest.add(copied as usize + k) = buffer[block_offset + k];
                        }
                        copied += chunk as u64;
                        current_blk_idx += 1;
                        block_offset = 0;
                    }
                    let zero_start = dest.add(len as usize);
                    let zero_len = ph.p_memsz - ph.p_filesz;
                    for k in 0..zero_len {
                        *zero_start.add(k as usize) = 0;
                    }
                }
            }
            let elf_total_pages = if elf_load_end > elf_load_base {
                ((elf_load_end - elf_load_base + 4095) / 4096) as usize
            } else {
                0
            };
            crate::serial::print_serial(b"[SHELL] Bytes at Entry (0x401000): ");
            let entry_ptr = 0x401000 as *const u8;
            for k in 0..8 {
                crate::serial::print_hex(*entry_ptr.add(k) as u64);
                crate::serial::print_serial(b" ");
            }
            crate::serial::print_serial(b"\n");
            if let Some(pid) =
                crate::task::spawn_process(elf_hdr.e_entry, args, crate::task::AbiType::Linux, elf_total_pages)
            {
                crate::task::FOREGROUND_TASK_ID = Some(pid);
                return true;
            }
        } else {
            let pages_needed = (inode.size as usize + 4095) / 4096;
            let base_page_opt = crate::pmm::allocate_contiguous_frames(pages_needed);

            if base_page_opt.is_none() {
                 print(b"Error: Insufficient contiguous memory!\n");
                 return false;
            }
            let base_page = base_page_opt.unwrap();

            crate::serial::print_serial(b"[SHELL] Binary Base Page (Contiguous): ");
            crate::serial::print_hex(base_page);
            crate::serial::print_serial(b"\n");

            if base_page == 0 {
                print(b"Error: Insufficient memory!\n");
                return false;
            }
            let mut buffer = [0u8; 4096];
            let dest_base = base_page as *mut u8;
            for i in 0..pages_needed {
                crate::fs::ramdisk::read_block((inode.block as usize) + i, &mut buffer);
                let offset = i * 4096;
                let dest = dest_base.add(offset);
                let remaining = inode.size as usize - offset;
                let copy_len = if remaining > 4096 { 4096 } else { remaining };
                for j in 0..copy_len {
                    *dest.add(j) = buffer[j];
                }
            }
            if let Some(_pid) =
                crate::task::spawn_process(base_page, args, crate::task::AbiType::Native, pages_needed)
            {
                return true;
            } else {
                print(b"Error: Cannot start process!\n");
            }
        }
    } else {
        print(b"Not a file, love!\n");
    }
    false
}
unsafe fn print_cwd(inode_idx: usize) {
    if inode_idx == ROOT_INODE {
        print(b"MOMMY/");
        return;
    }
    let inode = get_inode(inode_idx).unwrap();
    if inode.parent != inode_idx as u32 {
        print_cwd(inode.parent as usize);
    }
    let parent_inode = get_inode(inode.parent as usize).unwrap();
    if let Some(name) = find_name_in_dir(parent_inode, inode_idx as u32) {
        if inode.parent != ROOT_INODE as u32 {
            print(b"/");
        }
        print(&name);
    } else {
        print(b"???");
    }
    if is_dir(inode_idx) {
        print(b"/");
    }
}
unsafe fn find_name_in_dir(
    parent_inode: &crate::fs::inode::Inode,
    target_inode: u32,
) -> Option<[u8; 32]> {
    let mut buffer = [0u8; 4096];
    crate::fs::ramdisk::read_block(parent_inode.block as usize, &mut buffer);
    let entry_size = core::mem::size_of::<DirEntry>();
    let max_entries = 4096 / entry_size;
    let entries = core::slice::from_raw_parts(buffer.as_ptr() as *const DirEntry, max_entries);
    for i in 0..max_entries {
        if entries[i].name[0] != 0 && entries[i].inode_idx == target_inode {
            return Some(entries[i].name);
        }
    }
    None
}
unsafe fn find_entry_in_dir(parent_inode: &crate::fs::inode::Inode, name: &[u8], buffer: &mut [u8; 4096]) -> Option<u32> {
    crate::fs::directory::find_entry(parent_inode, name, buffer)
}
unsafe fn change_dir(path: &[u8]) {
    if path == b".." {
        let current = get_inode(CURRENT_DIR).unwrap();
        if CURRENT_DIR != ROOT_INODE {
            CURRENT_DIR = current.parent as usize;
        }
        return;
    }
    let current = get_inode(CURRENT_DIR).unwrap();
    let mut buffer = [0u8; 4096];
    if let Some(child_idx) = crate::fs::directory::find_entry(current, path, &mut buffer) {
        let child = get_inode(child_idx as usize).unwrap();
        if child.file_type == FileType::Directory {
            CURRENT_DIR = child_idx as usize;
        } else {
            print(b"Not a folder, love!\n");
        }
    } else {
        print(b"Not found, darling!\n");
    }
}
unsafe fn is_dir(inode_idx: usize) -> bool {
    let inode = get_inode(inode_idx).unwrap();
    inode.file_type == FileType::Directory
}
#[allow(dead_code)]

pub fn run_test_sequence() {
    unsafe {
        BUFFER_IDX = 0;
        for i in 0..256 {
            INPUT_BUFFER[i] = 0;
        }
        crate::serial::print_serial(b"[SHELL] Auto-Executing PING 10.0.2.2...\n");
        crate::net::ping_blocking([10, 0, 2, 2]);
        crate::serial::print_serial(b"[SHELL] Ping Finished.\n");
    }
}
fn parse_ip(s: &str) -> Option<[u8; 4]> {
    let mut parts = [0u8; 4];
    let mut idx = 0;
    for part in s.split('.') {
        if idx >= 4 {
            return None;
        }
        let mut val: u16 = 0;
        if part.len() == 0 {
            return None;
        }
        for b in part.bytes() {
            if b < b'0' || b > b'9' {
                return None;
            }
            val = val * 10 + (b - b'0') as u16;
            if val > 255 {
                return None;
            }
        }
        parts[idx] = val as u8;
        idx += 1;
    }
    if idx == 4 {
        Some(parts)
    } else {
        None
    }
}
