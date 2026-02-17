#[no_mangle]
pub extern "C" fn syscall_dispatcher(
    syscall_number: u64,
    arg1: u64,
    arg2: u64,
    arg3: u64,
    arg4: u64,
) -> u64 {
    match crate::task::get_current_abi() {
        crate::task::AbiType::Native => match syscall_number {
            0 => {
                crate::task::exit_current();
                0
            }
            1 => {
                unsafe {
                    let slice = core::slice::from_raw_parts(arg1 as *const u8, arg2 as usize);
                    crate::vga::print(slice);
                    crate::serial::print_serial(slice);
                }
                0
            }
            2 => {
                unsafe {
                    crate::vga::clear_screen();
                }
                0
            }
            3 => {
                crate::power::shutdown();
                0
            }
            4 => {
                crate::power::reboot();
                0
            }
            5 => {
                unsafe {
                    let mut path_len = arg2 as usize;
                    if path_len > 1024 {
                        path_len = 1024;
                    }
                    let path_slice = core::slice::from_raw_parts(arg1 as *const u8, path_len);
                    let out_buf = core::slice::from_raw_parts_mut(arg4 as *mut u8, 32);
                    let root = crate::fs::inode::get_inode(crate::fs::ROOT_INODE).unwrap();
                    let inode_idx = if path_slice == b"." {
                        let cwd = *(&raw const crate::shell::CURRENT_DIR) as u32;
                        unsafe {
                            crate::vga::print(b"[SYS] CWD: ");
                            crate::vga::print_u64_vga(cwd as u64);
                            crate::vga::print(b"\n");
                        }
                        cwd
                    } else if let Some(idx) = crate::fs::directory::find_entry(root, path_slice) {
                        idx
                    } else {
                        return 1;
                    };
                    let inode = crate::fs::inode::get_inode(inode_idx as usize).unwrap();
                    if inode.file_type == crate::fs::inode::FileType::Directory {
                        let buffer = &mut *(&raw mut crate::fs::FS_BUFFER);
                        crate::fs::ramdisk::read_block(inode.block as usize, buffer);
                        let entry_size = core::mem::size_of::<crate::fs::directory::DirEntry>();
                        let max_entries = 4096 / entry_size;
                        let entries = core::slice::from_raw_parts(
                            buffer.as_ptr() as *const crate::fs::directory::DirEntry,
                            max_entries,
                        );
                        let mut count = 0;
                        let target_index = arg3;
                        for i in 0..max_entries {
                            if entries[i].name[0] != 0 {
                                if count == target_index {
                                    for k in 0..32 {
                                        out_buf[k] = entries[i].name[k];
                                    }
                                    if let Some(child_inode) =
                                        crate::fs::inode::get_inode(entries[i].inode_idx as usize)
                                    {
                                        if child_inode.file_type
                                            == crate::fs::inode::FileType::Directory
                                        {
                                            return 2;
                                        } else {
                                            return 0;
                                        }
                                    }
                                    return 1;
                                }
                                count += 1;
                            }
                        }
                    }
                }
                1
            }
            6 => {
                unsafe {
                    crate::vga::set_color(arg1 as u8);
                }
                0
            }
            _ => 0,
        },
        crate::task::AbiType::Linux => match syscall_number {
            60 => {
                crate::task::exit_current();
                0
            }
            1 => {
                let fd = arg1;
                let buf = arg2;
                let len = arg3;
                if fd == 1 || fd == 2 {
                    unsafe {
                        let slice = core::slice::from_raw_parts(buf as *const u8, len as usize);
                        crate::vga::print(slice);
                        crate::serial::print_serial(slice);
                    }
                    len
                } else {
                    0xFFFFFFFFFFFFFFFE
                }
            }
            _ => {
                unsafe {
                    crate::serial::print_serial(b"[LINUX] Unimplemented Syscall: ");
                    crate::serial::print_hex(syscall_number);
                    crate::serial::print_serial(b"\n");
                }
                0
            }
        },
    }
}
