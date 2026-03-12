use crate::layout::BLOCK_SIZE;

pub const CACHE_SIZE: usize = 32;

#[derive(Clone, Copy)]

pub struct CacheEntry {
    pub block_id: u32,
    pub data: [u8; BLOCK_SIZE],
    pub valid: bool,
    pub dirty: bool,
    pub last_access: u64,
}

impl Default for CacheEntry {

    fn default() -> Self {
        Self {
            block_id: 0,
            data: [0; BLOCK_SIZE],
            valid: false,
            dirty: false,
            last_access: 0,
        }
    }
}

pub struct BlockCache<'a> {
    entries: [CacheEntry; CACHE_SIZE],
    clock: u64,

    raw_read: &'a dyn Fn(u32, &mut [u8]),
    raw_write: &'a dyn Fn(u32, &[u8]),
}

impl<'a> BlockCache<'a> {

    pub fn new(
        raw_read: &'a dyn Fn(u32, &mut [u8]),
        raw_write: &'a dyn Fn(u32, &[u8]),
    ) -> Self {
        Self {
            entries: [CacheEntry::default(); CACHE_SIZE],
            clock: 0,
            raw_read,
            raw_write,
        }
    }

    pub fn read_block(&mut self, block_id: u32, buf: &mut [u8]) {
        self.clock += 1;

        for i in 0..CACHE_SIZE {
            if self.entries[i].valid && self.entries[i].block_id == block_id {
                self.entries[i].last_access = self.clock;
                buf.copy_from_slice(&self.entries[i].data);
                return;
            }
        }

        let mut disk_data = [0u8; BLOCK_SIZE];
        (self.raw_read)(block_id, &mut disk_data);
        buf.copy_from_slice(&disk_data);

        self.insert_to_cache(block_id, disk_data, false);
    }

    pub fn write_block(&mut self, block_id: u32, buf: &[u8]) {
        self.clock += 1;

        let mut data = [0u8; BLOCK_SIZE];
        data.copy_from_slice(buf);

        for i in 0..CACHE_SIZE {
            if self.entries[i].valid && self.entries[i].block_id == block_id {
                self.entries[i].last_access = self.clock;
                self.entries[i].dirty = true;
                self.entries[i].data = data;
                return;
            }
        }

        self.insert_to_cache(block_id, data, true);
    }

    pub fn sync(&mut self) {
        for i in 0..CACHE_SIZE {
            if self.entries[i].valid && self.entries[i].dirty {
                (self.raw_write)(self.entries[i].block_id, &self.entries[i].data);
                self.entries[i].dirty = false;
            }
        }
    }

    fn insert_to_cache(&mut self, block_id: u32, data: [u8; BLOCK_SIZE], dirty: bool) {
        let mut lru_idx = 0;
        let mut oldest = u64::MAX;

        for i in 0..CACHE_SIZE {
            if !self.entries[i].valid {
                self.entries[i] = CacheEntry {
                    block_id,
                    data,
                    valid: true,
                    dirty,
                    last_access: self.clock,
                };
                return;
            }
            if self.entries[i].last_access < oldest {
                oldest = self.entries[i].last_access;
                lru_idx = i;
            }
        }

        if self.entries[lru_idx].dirty {

            (self.raw_write)(self.entries[lru_idx].block_id, &self.entries[lru_idx].data);
        }

        self.entries[lru_idx] = CacheEntry {
            block_id,
            data,
            valid: true,
            dirty,
            last_access: self.clock,
        };
    }
}
