use crate::layout::{Inode, Superblock, BLOCK_SIZE, MSW_MAGIC};

pub struct Mfs<'a> {
    pub superblock: Superblock,
    pub read_block: &'a dyn Fn(u32, &mut [u8]),
    pub write_block: &'a dyn Fn(u32, &[u8]),
    pub enc_key: Option<[u8; 32]>,
}

impl<'a> Mfs<'a> {

    pub fn new(
        read_block: &'a dyn Fn(u32, &mut [u8]),
        write_block: &'a dyn Fn(u32, &[u8]),
        enc_key: Option<[u8; 32]>,
    ) -> Result<Self, &'static str> {
        let mut sb_buf = [0u8; BLOCK_SIZE];
        read_block(0, &mut sb_buf);

        let sb = unsafe { core::ptr::read(sb_buf.as_ptr() as *const Superblock) };
        if sb.magic != MSW_MAGIC {
            return Err("Invalid MSW Magic");
        }

        Ok(Self {
            superblock: sb,
            read_block,
            write_block,
            enc_key,
        })
    }

    pub fn format(
        write_block: &dyn Fn(u32, &[u8]),
        total_blocks: u32,
        is_encrypted: u8,
        salt: [u8; 16],
        enc_key: Option<[u8; 32]>,
    ) -> Result<(), &'static str> {
        let total_inodes = total_blocks / 4;
        let inode_bitmap_start = 1;
        let inode_bitmap_blocks = (total_inodes + (BLOCK_SIZE as u32 * 8) - 1) / (BLOCK_SIZE as u32 * 8);

        let block_bitmap_start = inode_bitmap_start + inode_bitmap_blocks;
        let block_bitmap_blocks = (total_blocks + (BLOCK_SIZE as u32 * 8) - 1) / (BLOCK_SIZE as u32 * 8);

        let inode_table_start = block_bitmap_start + block_bitmap_blocks;
        let inodes_per_block = (BLOCK_SIZE / core::mem::size_of::<Inode>()) as u32;
        let inode_table_blocks = (total_inodes + inodes_per_block - 1) / inodes_per_block;

        let data_blocks_start = inode_table_start + inode_table_blocks;

        let sb = Superblock {
            magic: MSW_MAGIC,
            version: 1,
            block_size: BLOCK_SIZE as u32,
            total_blocks,
            total_inodes,
            inode_bitmap_start,
            block_bitmap_start,
            inode_table_start,
            data_blocks_start,
            is_encrypted,
            salt,
            padding: [0; 4043],
        };

        let mut buf = [0u8; BLOCK_SIZE];
        unsafe {
            let sb_ptr = &sb as *const Superblock as *const u8;
            core::ptr::copy_nonoverlapping(sb_ptr, buf.as_mut_ptr(), core::mem::size_of::<Superblock>());
        }
        write_block(0, &buf);

        let write_enc = |block_num: u32, buf: &mut [u8]| {
            if let Some(key) = &enc_key {
                crate::crypto::encrypt_block(key, block_num, buf);
            }
            write_block(block_num, buf);
        };

        for b in 0..inode_bitmap_blocks {
            buf.fill(0);
            if b == 0 { buf[0] = 1; }
            write_enc(inode_bitmap_start + b, &mut buf);
        }

        for b in 0..block_bitmap_blocks {
            buf.fill(0);
            let sys_used = data_blocks_start + 1;
            for i in 0..sys_used {
                if i / (BLOCK_SIZE as u32 * 8) == b {
                    let bit_idx = (i % (BLOCK_SIZE as u32 * 8)) as usize;
                    buf[bit_idx / 8] |= 1 << (bit_idx % 8);
                }
            }
            write_enc(block_bitmap_start + b, &mut buf);
        }

        let mut root_inode = Inode::empty();
        root_inode.mode = crate::layout::InodeType::Directory as u16 | 0x01FF;
        root_inode.links_count = 2;
        root_inode.direct_blocks[0] = data_blocks_start;

        for b in 0..inode_table_blocks {
            buf.fill(0);
            if b == 0 {
                unsafe {
                    let inode_ptr = &root_inode as *const Inode as *const u8;
                    core::ptr::copy_nonoverlapping(inode_ptr, buf.as_mut_ptr(), core::mem::size_of::<Inode>());
                }
            }
            write_enc(inode_table_start + b, &mut buf);
        }

        buf.fill(0);
        write_enc(data_blocks_start, &mut buf);

        Ok(())
    }

    pub fn read_data_block(&self, block_num: u32, buf: &mut [u8]) {
        (self.read_block)(block_num, buf);
        if let Some(engine) = &self.enc_key {
            if block_num != 0 {
                crate::crypto::decrypt_block(engine, block_num, buf);
            }
        }
    }

    pub fn write_data_block(&self, block_num: u32, buf: &[u8]) {
        let mut disk_buf = [0u8; BLOCK_SIZE];
        disk_buf.copy_from_slice(buf);

        if let Some(engine) = &self.enc_key {
            if block_num != 0 {
                crate::crypto::encrypt_block(engine, block_num, &mut disk_buf);
            }
        }
        (self.write_block)(block_num, &disk_buf);
    }

    pub fn allocate_block(&self) -> Option<u32> {
        let bitmap_start = self.superblock.block_bitmap_start;
        let total_blocks = self.superblock.total_blocks;
        let bitmap_blocks = (total_blocks + (BLOCK_SIZE as u32 * 8) - 1) / (BLOCK_SIZE as u32 * 8);

        for b in 0..bitmap_blocks {
            let mut buf = [0u8; BLOCK_SIZE];
            self.read_data_block(bitmap_start + b, &mut buf);

            for i in 0..BLOCK_SIZE {
                if buf[i] != 0xFF {
                    for bit in 0..8 {
                        if (buf[i] & (1 << bit)) == 0 {
                            let block_num = (b * BLOCK_SIZE as u32 * 8) + (i as u32 * 8) + bit;
                            if block_num >= total_blocks {
                                return None;
                            }
                            buf[i] |= 1 << bit;
                            self.write_data_block(bitmap_start + b, &buf);

                            let zero_buf = [0u8; BLOCK_SIZE];
                            self.write_data_block(block_num, &zero_buf);

                            return Some(block_num);
                        }
                    }
                }
            }
        }
        None
    }

    pub fn free_block(&self, block_num: u32) {
        let bitmap_start = self.superblock.block_bitmap_start;
        let b = block_num / (BLOCK_SIZE as u32 * 8);
        let bit_idx = block_num % (BLOCK_SIZE as u32 * 8);
        let byte_idx = (bit_idx / 8) as usize;
        let bit_offset = bit_idx % 8;

        let mut buf = [0u8; BLOCK_SIZE];
        self.read_data_block(bitmap_start + b, &mut buf);

        buf[byte_idx] &= !(1 << bit_offset);

        self.write_data_block(bitmap_start + b, &buf);
    }

    pub fn allocate_inode(&self) -> Option<u32> {
        let bitmap_start = self.superblock.inode_bitmap_start;
        let total_inodes = self.superblock.total_inodes;
        let bitmap_blocks = (total_inodes + (BLOCK_SIZE as u32 * 8) - 1) / (BLOCK_SIZE as u32 * 8);

        for b in 0..bitmap_blocks {
            let mut buf = [0u8; BLOCK_SIZE];
            self.read_data_block(bitmap_start + b, &mut buf);

            for i in 0..BLOCK_SIZE {
                if buf[i] != 0xFF {
                    for bit in 0..8 {
                        if (buf[i] & (1 << bit)) == 0 {
                            let inode_num = (b * BLOCK_SIZE as u32 * 8) + (i as u32 * 8) + bit;
                            if inode_num >= total_inodes {
                                return None;
                            }
                            buf[i] |= 1 << bit;
                            self.write_data_block(bitmap_start + b, &buf);
                            return Some(inode_num);
                        }
                    }
                }
            }
        }
        None
    }

    pub fn read_inode(&self, inode_num: u32) -> Option<Inode> {
        if inode_num >= self.superblock.total_inodes {
            return None;
        }
        let inodes_per_block = (BLOCK_SIZE / core::mem::size_of::<Inode>()) as u32;
        let block_offset = inode_num / inodes_per_block;
        let inode_index = (inode_num % inodes_per_block) as usize;

        let mut buf = [0u8; BLOCK_SIZE];
        self.read_data_block(self.superblock.inode_table_start + block_offset, &mut buf);

        let inode_ptr = buf.as_ptr() as *const Inode;
        let inode = unsafe { core::ptr::read(inode_ptr.add(inode_index)) };
        Some(inode)
    }

    pub fn write_inode(&self, inode_num: u32, inode: &Inode) {
        if inode_num >= self.superblock.total_inodes {
            return;
        }
        let inodes_per_block = (BLOCK_SIZE / core::mem::size_of::<Inode>()) as u32;
        let block_offset = inode_num / inodes_per_block;
        let inode_index = (inode_num % inodes_per_block) as usize;

        let mut buf = [0u8; BLOCK_SIZE];
        self.read_data_block(self.superblock.inode_table_start + block_offset, &mut buf);

        let inode_ptr = buf.as_mut_ptr() as *mut Inode;
        unsafe {
            core::ptr::write(inode_ptr.add(inode_index), *inode);
        }

        self.write_data_block(self.superblock.inode_table_start + block_offset, &buf);
    }
}
