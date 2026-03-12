use msw::crypto::{derive_key, encrypt_block};
use msw::layout::{Inode, InodeType, Superblock, BLOCK_SIZE, MSW_MAGIC};
use std::env;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::mem::size_of;

fn write_encrypted(
    file: &mut File,
    enc_key: &Option<[u8; 32]>,
    start_block: u32,
    blocks: u32,
    mut data: Vec<u8>,
) -> std::io::Result<()> {
    for b in 0..blocks {
        let block_num = start_block + b;
        let offset = (b as usize) * BLOCK_SIZE;
        let mut block_slice = [0u8; BLOCK_SIZE];
        block_slice.copy_from_slice(&data[offset..offset + BLOCK_SIZE]);

        if let Some(engine) = enc_key {
            encrypt_block(engine, block_num, &mut block_slice);
        }

        file.seek(SeekFrom::Start((block_num as u64) * (BLOCK_SIZE as u64)))?;
        file.write_all(&block_slice)?;
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 || args.len() > 4 {
        eprintln!(
            "Usage: {} <output_file> <size_in_mb> [passphrase]",
            args[0]
        );
        std::process::exit(1);
    }

    let outfile_path = &args[1];
    let size_mb: u32 = args[2].parse().expect("Invalid size specification");
    let passphrase = if args.len() == 4 { &args[3] } else { "mommy" };

    let total_blocks = (size_mb * 1024 * 1024) / (BLOCK_SIZE as u32);
    let total_inodes = total_blocks / 4;

    let inode_bitmap_start = 1;
    let inode_bitmap_blocks = (total_inodes + (BLOCK_SIZE as u32 * 8) - 1) / (BLOCK_SIZE as u32 * 8);

    let block_bitmap_start = inode_bitmap_start + inode_bitmap_blocks;
    let block_bitmap_blocks = (total_blocks + (BLOCK_SIZE as u32 * 8) - 1) / (BLOCK_SIZE as u32 * 8);

    let inode_table_start = block_bitmap_start + block_bitmap_blocks;
    let inodes_per_block = (BLOCK_SIZE / size_of::<Inode>()) as u32;
    let inode_table_blocks = (total_inodes + inodes_per_block - 1) / inodes_per_block;

    let data_blocks_start = inode_table_start + inode_table_blocks;

    println!("Formatting MSW with XTS-AES 256 Encryption:");
    println!("  Total Blocks: {}", total_blocks);
    println!("  Total Inodes: {}", total_inodes);
    println!(
        "  Inode Bitmap Blocks: {} (Starts at {})",
        inode_bitmap_blocks, inode_bitmap_start
    );
    println!(
        "  Block Bitmap Blocks: {} (Starts at {})",
        block_bitmap_blocks, block_bitmap_start
    );
    println!(
        "  Inode Table Blocks:  {} (Starts at {})",
        inode_table_blocks, inode_table_start
    );
    println!("  Data Blocks Start:   {}", data_blocks_start);

    let mut file = File::create(outfile_path)?;

    let mut salt = [0u8; 16];
    if let Ok(mut rnd) = File::open("/dev/urandom") {
        let _ = rnd.read_exact(&mut salt);
    }

    println!("  Deriving AES key from Passphrase using Argon2... Please wait.");
    let enc_key_engine = derive_key(passphrase, &salt);
    if enc_key_engine.is_none() {
        eprintln!("Fatal error during Argon2 key derivation. Allocation failed.");
        std::process::exit(1);
    }

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
        is_encrypted: 1,
        salt,
        padding: [0; 4043],
    };

    let mut sb_buf = vec![0u8; BLOCK_SIZE];
    unsafe {
        let sb_ptr = &sb as *const Superblock as *const u8;
        sb_buf[..size_of::<Superblock>()]
            .copy_from_slice(std::slice::from_raw_parts(sb_ptr, size_of::<Superblock>()));
    }
    file.write_all(&sb_buf)?;

    let mut inode_bitmap = vec![0u8; (inode_bitmap_blocks * BLOCK_SIZE as u32) as usize];
    inode_bitmap[0] = 1;
    write_encrypted(&mut file, &enc_key_engine, inode_bitmap_start, inode_bitmap_blocks, inode_bitmap)?;

    let mut block_bitmap = vec![0u8; (block_bitmap_blocks * BLOCK_SIZE as u32) as usize];
    let system_blocks_used = data_blocks_start + 1;
    for i in 0..system_blocks_used {
        let byte_idx = (i / 8) as usize;
        let bit_idx = i % 8;
        block_bitmap[byte_idx] |= 1 << bit_idx;
    }
    write_encrypted(&mut file, &enc_key_engine, block_bitmap_start, block_bitmap_blocks, block_bitmap)?;

    let mut root_inode = Inode {
        mode: msw::layout::InodeType::Directory as u16 | 0x01FF,
        links_count: 2,
        uid: 0,
        gid: 0,
        size: 0,
        access_time: 0,
        creation_time: 0,
        modification_time: 0,
        flags: 0,
        direct_blocks: [data_blocks_start, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        indirect_block: 0,
        double_indirect_block: 0,
        created_at: 0,
        modified_at: 0,
        padding: [0; 44],
    };
    root_inode.direct_blocks[0] = data_blocks_start;

    let mut inode_table = vec![0u8; (inode_table_blocks * BLOCK_SIZE as u32) as usize];
    unsafe {
        let inode_ptr = &root_inode as *const Inode as *const u8;
        inode_table[..size_of::<Inode>()]
            .copy_from_slice(std::slice::from_raw_parts(inode_ptr, size_of::<Inode>()));
    }
    write_encrypted(&mut file, &enc_key_engine, inode_table_start, inode_table_blocks, inode_table)?;

    let empty_block = vec![0u8; BLOCK_SIZE];
    write_encrypted(&mut file, &enc_key_engine, data_blocks_start, 1, empty_block)?;

    let final_pos = (total_blocks as u64) * (BLOCK_SIZE as u64) - 1;
    file.seek(SeekFrom::Start(final_pos))?;
    file.write_all(&[0])?;

    println!("MSW Image created successfully at '{}'", outfile_path);
    Ok(())
}
