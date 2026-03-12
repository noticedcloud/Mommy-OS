use crate::core::Mfs;
use crate::layout::{DirEntry, BLOCK_SIZE};
use core::mem::size_of;

pub fn lookup(msw: &Mfs, dir_inode_num: u32, name: &str) -> Option<u32> {
    let dir_inode = msw.read_inode(dir_inode_num)?;

    let entries_per_block = BLOCK_SIZE / size_of::<DirEntry>();
    let name_bytes = name.as_bytes();

    let direct_blocks = dir_inode.direct_blocks;

    for &block_num in &direct_blocks {
        if block_num == 0 {
            continue;
        }

        let mut buf = [0u8; BLOCK_SIZE];
        msw.read_data_block(block_num, &mut buf);

        let entries = unsafe {
            core::slice::from_raw_parts(buf.as_ptr() as *const DirEntry, entries_per_block)
        };

        for entry in entries {
            if entry.inode == 0 {
                continue;
            }

            let len = entry.name_len as usize;
            if len == name_bytes.len() && &entry.name[..len] == name_bytes {
                return Some(entry.inode);
            }
        }
    }

    None
}

pub fn resolve_path(msw: &Mfs, path: &str) -> Option<u32> {
    let mut current_inode = 0;

    for part in path.split('/') {
        if part.is_empty() || part == "." {
            continue;
        }

        if part == ".." {

            continue;
        }

        match lookup(msw, current_inode, part) {
            Some(inode) => current_inode = inode,
            None => return None,
        }
    }

    Some(current_inode)
}
