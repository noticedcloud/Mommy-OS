use crate::core::Mfs;
use crate::layout::BLOCK_SIZE;
use core::cmp::min;

pub fn read_file(msw: &Mfs, inode_num: u32, offset: u32, buf: &mut [u8]) -> Option<usize> {
    let inode = msw.read_inode(inode_num)?;

    if offset >= inode.size {
        return Some(0);
    }

    let mut bytes_read = 0;
    let mut current_offset = offset;
    let to_read = min(buf.len() as u32, inode.size - offset) as usize;

    let direct_blocks = inode.direct_blocks;

    while bytes_read < to_read {
        let block_idx = (current_offset / BLOCK_SIZE as u32) as usize;
        let block_offset = (current_offset % BLOCK_SIZE as u32) as usize;

        let block_num = if block_idx < 12 {
            direct_blocks[block_idx]
        } else {

            return Some(bytes_read);
        };

        if block_num == 0 {
            break;
        }

        let mut disk_buf = [0u8; BLOCK_SIZE];
        msw.read_data_block(block_num, &mut disk_buf);

        let chunk_size = min(BLOCK_SIZE - block_offset, to_read - bytes_read);
        buf[bytes_read..bytes_read + chunk_size].copy_from_slice(&disk_buf[block_offset..block_offset + chunk_size]);

        bytes_read += chunk_size;
        current_offset += chunk_size as u32;
    }

    Some(bytes_read)
}

pub fn write_file_cow(msw: &Mfs, inode_num: u32, offset: u32, data: &[u8]) -> Option<usize> {
    let mut inode = msw.read_inode(inode_num)?;
    let mut bytes_written = 0;
    let mut current_offset = offset;
    let to_write = data.len();

    while bytes_written < to_write {
        let block_idx = (current_offset / BLOCK_SIZE as u32) as usize;
        let block_offset = (current_offset % BLOCK_SIZE as u32) as usize;

        if block_idx >= 12 {
            break;
        }

        let old_block_num = inode.direct_blocks[block_idx];

        let new_block_num = msw.allocate_block()?;

        let mut disk_buf = [0u8; BLOCK_SIZE];

        if old_block_num != 0 && (block_offset > 0 || to_write - bytes_written < BLOCK_SIZE) {
            msw.read_data_block(old_block_num, &mut disk_buf);
        }

        let chunk_size = min(BLOCK_SIZE - block_offset, to_write - bytes_written);
        disk_buf[block_offset..block_offset + chunk_size].copy_from_slice(&data[bytes_written..bytes_written + chunk_size]);

        msw.write_data_block(new_block_num, &disk_buf);

        inode.direct_blocks[block_idx] = new_block_num;

        if old_block_num != 0 {
            msw.free_block(old_block_num);
        }

        bytes_written += chunk_size;
        current_offset += chunk_size as u32;

        if current_offset > inode.size {
            inode.size = current_offset;
        }
    }

    msw.write_inode(inode_num, &inode);

    Some(bytes_written)
}
