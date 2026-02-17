#[derive(Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum FileType {
    Free = 0,
    File = 1,
    Directory = 2,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Inode {
    pub id: u32,
    pub file_type: FileType,
    pub size: u32,
    pub block: u32,
    pub parent: u32,
}
const MAX_INODES: usize = 128;
static mut INODE_TABLE: [Inode; MAX_INODES] = [Inode {
    id: 0,
    file_type: FileType::Free,
    size: 0,
    block: 0,
    parent: 0,
}; MAX_INODES];
pub fn init_inodes() {
    unsafe {
        for i in 0..MAX_INODES {
            INODE_TABLE[i] = Inode {
                id: i as u32,
                file_type: FileType::Free,
                size: 0,
                block: 0,
                parent: 0,
            };
        }
    }
}
pub fn allocate_inode() -> Option<usize> {
    unsafe {
        for i in 0..MAX_INODES {
            if INODE_TABLE[i].file_type == FileType::Free {
                return Some(i);
            }
        }
    }
    None
}
pub fn get_inode(index: usize) -> Option<&'static mut Inode> {
    unsafe {
        if index < MAX_INODES {
            Some(&mut INODE_TABLE[index])
        } else {
            None
        }
    }
}
