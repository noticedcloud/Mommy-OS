use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::mem;
#[derive(Copy, Clone)]
#[repr(C, packed)]
struct MomHeader {
    magic: [u8; 4],
    version: u8,
    type_flag: u8,
    compression: u8,
    padding: u8,
    capabilities: u64,
    entry_offset: u64,
    total_size: u64,
    stack_request: u64,
    checksum: u64,
    reserved: [u8; 16],
}
impl Default for MomHeader {
    fn default() -> Self {
        Self {
            magic: [0x4D, 0x4F, 0x4D, 0x21],
            version: 1,
            type_flag: 0,
            compression: 0,
            padding: 0,
            capabilities: 0,
            entry_offset: 64,
            total_size: 0,
            stack_request: 0,
            checksum: 0,
            reserved: [0; 16],
        }
    }
}
fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: packer <input_raw_bin> <output_mom_file>");
        std::process::exit(1);
    }
    let input_path = &args[1];
    let output_path = &args[2];
    let mut file_in = File::open(input_path)?;
    let mut buffer = Vec::new();
    file_in.read_to_end(&mut buffer)?;
    let mut header = MomHeader::default();
    header.total_size = (mem::size_of::<MomHeader>() + buffer.len()) as u64;
    let mut sum: u64 = 0;
    for &byte in &buffer {
        sum = sum.wrapping_add(byte as u64);
    }
    header.checksum = sum;
    let total_size = header.total_size;
    let checksum = header.checksum;
    let entry_offset = header.entry_offset;
    let header_bytes: [u8; 64] = unsafe { mem::transmute(header) };
    let mut file_out = File::create(output_path)?;
    file_out.write_all(&header_bytes)?;
    file_out.write_all(&buffer)?;
    println!(
        "Packed '{}' -> '{}'. Size: {}, Checksum: {}, Entry: {}",
        input_path, output_path, total_size, checksum, entry_offset
    );
    Ok(())
}
