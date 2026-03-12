pub const BLOCK_SIZE: usize = 4096;
pub const MSW_MAGIC: [u8; 4] = *b"MOM!";

#[repr(C, packed)]

#[derive(Debug, Clone, Copy)]

pub struct Superblock {
    pub magic: [u8; 4],
    pub version: u32,
    pub block_size: u32,
    pub total_blocks: u32,
    pub total_inodes: u32,
    pub inode_bitmap_start: u32,
    pub block_bitmap_start: u32,
    pub inode_table_start: u32,
    pub data_blocks_start: u32,
    pub is_encrypted: u8,
    pub salt: [u8; 16],
    pub padding: [u8; 4043],
}

#[repr(u16)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]

pub enum InodeType {
    Unknown = 0,
    File = 1,
    Directory = 2,
    Symlink = 3,
}

#[repr(C, packed)]

#[derive(Debug, Clone, Copy)]

pub struct Inode {

    pub mode: u16,
    pub uid: u16,
    pub gid: u16,
    pub links_count: u16,
    pub size: u32,
    pub direct_blocks: [u32; 12],
    pub indirect_block: u32,
    pub double_indirect_block: u32,
    pub created_at: u64,

    pub modified_at: u64,
    pub access_time: u64,
    pub creation_time: u64,

    pub modification_time: u64,
    pub flags: u32,

    pub padding: [u8; 44],
}

impl Inode {
    pub const fn empty() -> Self {
        Self {
            mode: 0,
            links_count: 0,
            uid: 0,
            gid: 0,
            size: 0,
            access_time: 0,
            creation_time: 0,
            modification_time: 0,
            flags: 0,
            direct_blocks: [0; 12],
            indirect_block: 0,
            double_indirect_block: 0,
            created_at: 0,
            modified_at: 0,
            padding: [0; 44],
        }
    }
}

#[repr(C, packed)]

#[derive(Debug, Clone, Copy)]

pub struct DirEntry {
    pub inode: u32,
    pub name_len: u8,
    pub entry_type: u8,
    pub padding: u16,
    pub name: [u8; 56],
}