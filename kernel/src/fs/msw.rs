use msw::core::Mfs;

pub const MSW_MAX_BLOCKS: usize = 2560;
static mut MSW_STORAGE: [usize; MSW_MAX_BLOCKS] = [0; MSW_MAX_BLOCKS];

pub fn init_msw_ramdisk() {
    unsafe {
        for i in 0..MSW_MAX_BLOCKS {
            let (frame, _) = crate::pmm::allocate_frame_type(crate::pmm::MemoryType::Kernel).expect("OOM MSW");
            MSW_STORAGE[i] = frame as usize;
        }
    }
}

pub fn msw_read_block(block_index: usize, buffer: &mut [u8]) {
    unsafe {
        if block_index >= MSW_MAX_BLOCKS { return; }
        let ptr = MSW_STORAGE[block_index] as *const u8;
        for i in 0..4096 { buffer[i] = *ptr.add(i); }
    }
}

pub fn msw_write_block(block_index: usize, buffer: &[u8]) {
    unsafe {
        if block_index >= MSW_MAX_BLOCKS { return; }
        let ptr = MSW_STORAGE[block_index] as *mut u8;
        for i in 0..4096 {
            if i < buffer.len() { *ptr.add(i) = buffer[i]; } else { *ptr.add(i) = 0; }
        }
    }
}

pub struct KernelMfs {}

fn get_rdtsc_entropy() -> [u8; 16] {
    let mut salt = [0u8; 16];
    for i in 0..16 {
        let mut a: u32;
        unsafe {
            core::arch::asm!("rdtsc", out("eax") a, out("edx") _, options(nomem, nostack, preserves_flags));
        }
        salt[i] = (a & 0xFF) as u8;

        for _ in 0..5000 { unsafe { core::arch::asm!("nop") }; }
    }
    salt
}

impl KernelMfs {

    pub fn init() -> Result<(), &'static str> {
        init_msw_ramdisk();

        let mut sb_buf = [0u8; 4096];
        unsafe { msw_read_block(0, &mut sb_buf); }
        let pre_sb = unsafe { core::ptr::read(sb_buf.as_ptr() as *const msw::layout::Superblock) };

        let mut enc_key = None;

        if pre_sb.magic != msw::layout::MSW_MAGIC {
            crate::vga::print(b"\n[MSW] Unformatted drive detected. Format it? (y/n): ");
            loop {
                let ch = crate::keyboard::get_char_polling();
                if ch == b'y' || ch == b'Y' {
                    crate::vga::print(b"y\n");
                    break;
                } else if ch == b'n' || ch == b'N' {
                    crate::vga::print(b"n\n[MSW] Mount aborted.\n");
                    return Err("Unformatted drive");
                }
            }

            crate::vga::print(b"[MSW] Encrypt whole disk? (y/n): ");
            let mut is_enc = 0;
            let mut salt = [0u8; 16];
            loop {
                let ch = crate::keyboard::get_char_polling();
                if ch == b'y' || ch == b'Y' {
                    crate::vga::print(b"y\n");
                    is_enc = 1;
                    crate::vga::print(b"[MSW] Enter new passphrase: ");
                    let mut pass_buf = [0u8; 32];
                    let len = crate::keyboard::read_line_polling(&mut pass_buf, true);
                    let passphrase_str = core::str::from_utf8(&pass_buf[..len]).unwrap_or("mommy");

                    crate::vga::print(b"\n[MSW] Generating Salt via RDTSC entropy...\n");
                    salt = get_rdtsc_entropy();

                    crate::vga::print(b"[MSW] Deriving AES Key (Argon2)... Please wait... ");
                    enc_key = msw::crypto::derive_key(passphrase_str, &salt);
                    if enc_key.is_none() {
                        crate::vga::print(b"[FAIL]\n");
                        return Err("Key Derivation Failed");
                    }
                    crate::vga::print(b"[OK]\n");
                    break;
                } else if ch == b'n' || ch == b'N' {
                    crate::vga::print(b"n\n");
                    break;
                }
            }

            crate::vga::print(b"[MSW] Formatting disk... ");

            msw::core::Mfs::format(
                &|block_num, buf| {
                    msw_write_block(block_num as usize, buf);
                },
                2560,
                is_enc,
                salt,
                enc_key
            )?;
            crate::vga::print(b"[OK]\n");

        } else {

            if pre_sb.is_encrypted == 1 {
                crate::vga::print(b"\n[MSW] Disk is Locked. Enter passphrase: ");
                let mut pass_buf = [0u8; 32];
                let len = crate::keyboard::read_line_polling(&mut pass_buf, true);
                let passphrase_str = core::str::from_utf8(&pass_buf[..len]).unwrap_or("");

                crate::vga::print(b"\n[MSW] Unlocking (Argon2)... ");
                enc_key = msw::crypto::derive_key(passphrase_str, &pre_sb.salt);
                if enc_key.is_none() {
                    crate::vga::print(b"[FAIL]\n");
                    return Err("Wrong passphrase or OOM");
                }
                crate::vga::print(b"[OK]\n");
            }
        }

        let msw = Mfs::new(
            &|block_num, buf| { msw_read_block(block_num as usize, buf); },
            &|block_num, buf| { msw_write_block(block_num as usize, buf); },
            enc_key
        )?;

        crate::serial::print_serial(b"[MSW] Mounted successfully.\n");
        crate::serial::print_serial(b"      Total Blocks: ");
        crate::serial::print_hex(msw.superblock.total_blocks as u64);
        crate::serial::print_serial(b"\n");

        if let Some(root_inode) = msw.read_inode(0) {
            crate::serial::print_serial(b"[MSW] Root Inode size: ");
            crate::serial::print_hex(root_inode.size as u64);
            crate::serial::print_serial(b"\n");
        } else {
            return Err("Failed to read Root Inode");
        }

        Ok(())
    }
}
