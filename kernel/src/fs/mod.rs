pub mod directory;
pub mod inode;
pub mod ramdisk;
use crate::fs::directory::{add_entry, create_root_dir};
use crate::fs::inode::{allocate_inode, get_inode, init_inodes, FileType};
use crate::fs::ramdisk::{allocate_block, init_ramdisk, write_block};
pub use crate::vga::print;
pub const ROOT_INODE: usize = 0;
pub static mut FS_BUFFER: [u8; 4096] = [0; 4096];
fn create_dir(parent_idx: usize, name: &[u8]) -> usize {
    let inode_idx = allocate_inode().expect("No inodes");
    let block = allocate_block().expect("No blocks");
    {
        let inode = get_inode(inode_idx).unwrap();
        create_root_dir(inode, block as u32);
        inode.parent = parent_idx as u32;
    }
    {
        let parent = get_inode(parent_idx).unwrap();
        add_entry(parent, name, inode_idx as u32);
    }
    inode_idx
}
fn create_file(parent_idx: usize, name: &[u8]) {
    create_file_content(parent_idx, name, &[]);
}
fn create_file_content(parent_idx: usize, name: &[u8], content: &[u8]) {
    let inode_idx = allocate_inode().expect("No inodes");
    let needed_blocks = (content.len() + 4095) / 4096;
    let start_block = allocate_block().expect("No blocks");
    for _ in 0..(needed_blocks.saturating_sub(1)) {
        allocate_block().expect("No blocks for huge file");
    }
    {
        let inode = get_inode(inode_idx).unwrap();
        inode.file_type = FileType::File;
        inode.size = content.len() as u32;
        inode.block = start_block as u32;
        inode.parent = parent_idx as u32;
    }
    if content.len() > 0 {
        let mut chunks = content.chunks(4096);
        let mut current_block = start_block;
        while let Some(chunk) = chunks.next() {
            write_block(current_block, chunk);
            current_block += 1;
        }
    }
    {
        let parent = get_inode(parent_idx).unwrap();
        add_entry(parent, name, inode_idx as u32);
    }
}
pub fn init() {
    print(b"Initializing MommyFS...\n");
    init_ramdisk();
    init_inodes();
    let root_inode_idx = allocate_inode().expect("Failed to allocate root inode");
    let root_block = allocate_block().expect("Failed to allocate root block");
    let root_inode = get_inode(root_inode_idx).unwrap();
    create_root_dir(root_inode, root_block as u32);
    create_root_dir(root_inode, root_block as u32);
    if root_inode_idx != ROOT_INODE {
        panic!("Root inode allocated at wrong index!");
    }
    let verify = get_inode(root_inode_idx).unwrap();
    if verify.file_type == FileType::Directory {
        print(b"[DEBUG] Root created successfully (Type: Directory)\n");
    } else {
        print(b"[DEBUG] Root creation FAILED! Type is: ");
        let v = verify.file_type as u8;
        crate::vga::print_u64_vga(v as u64);
        print(b"\n");
    }
    print(b"Root 'MOMMY/' created.\n");
    let heart = create_dir(root_inode_idx, b"HEART");
    create_file(heart, b"PULSE");
    create_file(heart, b"RULES");
    create_file(heart, b"KISSES");
    let lap = create_dir(root_inode_idx, b"LAP");
    let user = create_dir(lap, b"darling");
    let toys = create_dir(user, b"TOYS");
    create_dir(user, b"SECRETS");
    create_dir(user, b"CUDDLES");
    create_dir(lap, b"GUEST");
    let chest = create_dir(root_inode_idx, b"CHEST");
    create_dir(chest, b"MILK");
    create_dir(chest, b"BONES");
    let whip = create_dir(chest, b"WHIP");
    let thighs = create_dir(root_inode_idx, b"THIGHS");
    create_dir(thighs, b"KEYS");
    create_dir(thighs, b"EYES");
    create_dir(thighs, b"VOICE");
    let belly = create_dir(root_inode_idx, b"BELLY");
    create_dir(belly, b"TUMMYACHES");
    create_dir(belly, b"SNACKS");
    let backpack = create_dir(root_inode_idx, b"BACKPACK");
    create_dir(backpack, b"STORIES");
    create_dir(backpack, b"TREASURES");
    let naughty = create_dir(root_inode_idx, b"NAUGHTY");
    create_dir(naughty, b"WHISPERS");
    create_dir(naughty, b"PUNISHMENTS");
    create_file_content(
        toys,
        b"hello.mom",
        include_bytes!("../../../userspace/hello.mom"),
    );
    let linux_test_content = include_bytes!("../../../userspace/linux_test");
    crate::serial::print_serial(b"[FS] linux_test SIZE: ");
    crate::serial::print_hex(linux_test_content.len() as u64);
    crate::serial::print_serial(b"\n");
    create_file_content(toys, b"linux_test", linux_test_content);
    crate::serial::print_serial(b"[FS] linux_test bytes: ");
    for i in 0..16 {
        crate::serial::print_hex(linux_test_content[i] as u64);
        crate::serial::print_serial(b" ");
    }
    crate::serial::print_serial(b"\n");
    create_file_content(whip, b"ls.mom", include_bytes!("../../../userspace/ls.mom"));
    create_file_content(
        whip,
        b"clear.mom",
        include_bytes!("../../../userspace/clear.mom"),
    );
    create_file_content(
        whip,
        b"echo.mom",
        include_bytes!("../../../userspace/echo.mom"),
    );
    create_file_content(
        whip,
        b"reboot.mom",
        include_bytes!("../../../userspace/reboot.mom"),
    );
    create_file_content(
        whip,
        b"shutdown.mom",
        include_bytes!("../../../userspace/shutdown.mom"),
    );
    print(b"Mommy's body structures initialized.\n");
}
pub fn verify_fs() {
    print(b"--- FS VERIFICATION ---\n");
    unsafe {
        let root = get_inode(ROOT_INODE).unwrap();
        print(b"ROOT INODE: ID=");
        crate::vga::print_u64_vga(root.id as u64);
        print(b" Type=");
        crate::vga::print_u64_vga(root.file_type as u64);
        print(b"\n");
        let buffer = &mut *(&raw mut FS_BUFFER);
        use crate::fs::directory::DirEntry;
        use crate::fs::ramdisk::read_block;
        read_block(root.block as usize, buffer);
        let max_entries = 4096 / core::mem::size_of::<DirEntry>();
        let entries = core::slice::from_raw_parts(buffer.as_ptr() as *const DirEntry, max_entries);
        for i in 0..max_entries {
            if entries[i].name[0] != 0 {
                print(b"  ENTRY: ");
                print(&entries[i].name);
                print(b" -> Inode ");
                crate::vga::print_u64_vga(entries[i].inode_idx as u64);
                let child = get_inode(entries[i].inode_idx as usize).unwrap();
                print(b" (Type=");
                crate::vga::print_u64_vga(child.file_type as u64);
                print(b")\n");
            }
        }
    }
    print(b"-----------------------\n");
}
