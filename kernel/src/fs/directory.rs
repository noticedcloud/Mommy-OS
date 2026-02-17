use crate::fs::inode::{FileType, Inode};
use crate::fs::ramdisk::{read_block, write_block};
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DirEntry {
    pub inode_idx: u32,
    pub name: [u8; 32],
}
pub fn create_root_dir(inode: &mut Inode, block_idx: u32) {
    inode.file_type = FileType::Directory;
    inode.size = 0;
    inode.block = block_idx;
    inode.parent = 0;
    let zeros = [0u8; 4096];
    write_block(block_idx as usize, &zeros);
}
pub fn add_entry(parent_inode: &mut Inode, name: &[u8], child_inode_idx: u32) -> bool {
    unsafe {
        let buffer = &mut *(&raw mut crate::fs::FS_BUFFER);
        read_block(parent_inode.block as usize, buffer);
        let entry_size = core::mem::size_of::<DirEntry>();
        let max_entries = 4096 / entry_size;
        let entries =
            core::slice::from_raw_parts_mut(buffer.as_mut_ptr() as *mut DirEntry, max_entries);
        for i in 0..max_entries {
            if entries[i].name[0] == 0 {
                entries[i].inode_idx = child_inode_idx;
                for j in 0..32 {
                    if j < name.len() {
                        entries[i].name[j] = name[j];
                    } else {
                        entries[i].name[j] = 0;
                    }
                }
                write_block(parent_inode.block as usize, buffer);
                crate::serial::print_serial(b"[FS] Added entry: ");
                crate::serial::print_serial(name);
                crate::serial::print_serial(b" (Inode: ");
                crate::serial::print_hex(child_inode_idx as u64);
                crate::serial::print_serial(b")\n");
                return true;
            }
        }
    }
    crate::serial::print_serial(b"[FS] Failed to add entry (Directory full?)\n");
    false
}
pub fn find_entry(parent_inode: &Inode, name: &[u8]) -> Option<u32> {
    unsafe {
        let buffer = &mut *(&raw mut crate::fs::FS_BUFFER);
        read_block(parent_inode.block as usize, buffer);
        let entry_size = core::mem::size_of::<DirEntry>();
        let max_entries = 4096 / entry_size;
        let entries = core::slice::from_raw_parts(buffer.as_ptr() as *const DirEntry, max_entries);
        for i in 0..max_entries {
            if entries[i].name[0] != 0 {
                let mut match_found = true;
                for j in 0..32 {
                    let mut c1 = entries[i].name[j];
                    let mut c2 = if j < name.len() { name[j] } else { 0 };
                    if c1 >= b'A' && c1 <= b'Z' {
                        c1 += 32;
                    }
                    if c2 >= b'A' && c2 <= b'Z' {
                        c2 += 32;
                    }
                    if c1 != c2 {
                        match_found = false;
                        break;
                    }
                    if c1 == 0 {
                        break;
                    }
                }
                if match_found {
                    return Some(entries[i].inode_idx);
                }
            }
        }
    }
    None
}
