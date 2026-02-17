use crate::drivers::pci::{get_bar0, pci_read};
use crate::pmm::allocate_frame;
use crate::vga::print;
use core::arch::asm;
const REG_CTRL: usize = 0x0000;
const REG_STATUS: usize = 0x0008;
const REG_EEPROM: usize = 0x0014;
const REG_ICR: usize = 0x00C0;
const REG_IMS: usize = 0x00D0;
const REG_IMC: usize = 0x00D8;
const REG_RCTL: usize = 0x0100;
const REG_TCTL: usize = 0x0400;
const REG_RDBAL: usize = 0x2800;
const REG_RDBAH: usize = 0x2804;
const REG_RDLEN: usize = 0x2808;
const REG_RDH: usize = 0x2810;
const REG_RDT: usize = 0x2818;
const REG_TDBAL: usize = 0x3800;
const REG_TDBAH: usize = 0x3804;
const REG_TDLEN: usize = 0x3808;
const REG_TDH: usize = 0x3810;
const REG_TDT: usize = 0x3818;
const REG_RAL: usize = 0x5400;
const REG_RAH: usize = 0x5404;
const CTRL_RST: u32 = 1 << 26;
const CTRL_SLU: u32 = 1 << 6;
const RCTL_EN: u32 = 1 << 1;
const RCTL_SBP: u32 = 1 << 2;
const RCTL_UPE: u32 = 1 << 3;
const RCTL_MPE: u32 = 1 << 4;
const RCTL_LPE: u32 = 1 << 5;
const RCTL_BAM: u32 = 1 << 15;
const TCTL_EN: u32 = 1 << 1;
const TCTL_PSP: u32 = 1 << 3;
const TCTL_CT: u32 = 0x0F << 4;
const TCTL_COLD: u32 = 0x40 << 12;
const CMD_EOP: u8 = 1 << 0;
const CMD_IFCS: u8 = 1 << 1;
#[repr(C, packed)]
struct TxDesc {
    addr: u64,
    length: u16,
    cso: u8,
    cmd: u8,
    status: u8,
    css: u8,
    special: u16,
}
#[repr(C, packed)]
struct RxDesc {
    addr: u64,
    length: u16,
    checksum: u16,
    status: u8,
    errors: u8,
    special: u16,
}
static mut MMIO_BASE: u32 = 0;
static mut TX_DESCS: *mut TxDesc = 0 as *mut TxDesc;
static mut TX_BUFFERS: *mut u8 = 0 as *mut u8;
static mut TX_TAIL: usize = 0;
static mut RX_DESCS: *mut RxDesc = 0 as *mut RxDesc;
static mut RX_BUFFERS: *mut u8 = 0 as *mut u8;
static mut RX_CUR: usize = 0;
const NUM_TX_DESCS: usize = 8;
const NUM_RX_DESCS: usize = 32;
pub unsafe fn init_e1000() {
    print(b"[NET] Scanning for Intel E1000...\n");
    let mut bus_found = 0;
    let mut slot_found = 0;
    let mut found = false;
    for bus in 0..256 {
        for slot in 0..32 {
            let vendor = pci_read(bus as u8, slot as u8, 0, 0) & 0xFFFF;
            let device = (pci_read(bus as u8, slot as u8, 0, 0) >> 16) & 0xFFFF;
            if vendor == 0x8086 && device == 0x100E {
                bus_found = bus;
                slot_found = slot;
                found = true;
                break;
            }
        }
        if found {
            break;
        }
    }
    if !found {
        print(b"[NET] E1000 Not Found!\n");
        return;
    }
    print(b"[NET] E1000 Found at ");
    crate::vga::print_u64_vga(bus_found as u64);
    print(b":");
    crate::vga::print_u64_vga(slot_found as u64);
    print(b"\n");
    use crate::drivers::pci::pci_write;
    let command_reg = pci_read(bus_found as u8, slot_found as u8, 0, 0x04);
    print(b"[NET] PCI Command: 0x");
    crate::serial::print_hex(command_reg as u64);
    print(b"\n");
    if (command_reg & 0x04) == 0 {
        print(b"[NET] Enabling Bus Mastering...\n");
        pci_write(
            bus_found as u8,
            slot_found as u8,
            0,
            0x04,
            command_reg | 0x04,
        );
    }
    MMIO_BASE = get_bar0(bus_found as u8, slot_found as u8, 0);
    print(b"[NET] MMIO Base: 0x");
    crate::serial::print_hex(MMIO_BASE as u64);
    print(b"\n");
    unsafe {
        let base = MMIO_BASE as u64;
        let size = 0x20000;
        for offset in (0..size).step_by(4096) {
            crate::paging::identity_map(
                base + offset,
                crate::paging::PAGE_WRITE | crate::paging::PAGE_NO_CACHE,
            );
        }
        print(b"[NET] MMIO Mapped.\n");
    }
    write_reg(REG_CTRL, CTRL_RST);
    for _ in 0..100000 {
        asm!("nop");
    }
    write_reg(REG_IMC, 0xFFFFFFFF);
    let mac_low: u32 = 0x12005452;
    let mac_high: u32 = 0x5634 | 0x80000000;
    write_reg(REG_RAL, mac_low);
    write_reg(REG_RAH, mac_high);
    let rx_desc_page = allocate_frame();
    RX_DESCS = rx_desc_page as *mut RxDesc;
    print(b"[NET] RX Desc Base: ");
    crate::vga::print_u64_vga(rx_desc_page);
    print(b"\n");
    for i in 0..NUM_RX_DESCS {
        let desc = &mut *RX_DESCS.add(i);
        let buf_addr = allocate_frame();
        desc.addr = buf_addr as u64;
        desc.status = 0;
    }
    write_reg(REG_RDBAL, rx_desc_page as u32);
    write_reg(REG_RDBAH, 0);
    write_reg(
        REG_RDLEN,
        (NUM_RX_DESCS * core::mem::size_of::<RxDesc>()) as u32,
    );
    write_reg(REG_RDH, 0);
    write_reg(REG_RDT, (NUM_RX_DESCS - 1) as u32);
    write_reg(
        REG_RCTL,
        RCTL_EN | RCTL_SBP | RCTL_UPE | RCTL_MPE | RCTL_BAM,
    );
    let desc_page = allocate_frame();
    TX_DESCS = desc_page as *mut TxDesc;
    print(b"[NET] TX Desc Base: ");
    crate::vga::print_u64_vga(desc_page);
    print(b"\n");
    let tx_pages = (NUM_TX_DESCS * 2048 + 4095) / 4096;
    let buf_page = allocate_frame();
    for _ in 0..tx_pages - 1 {
        allocate_frame();
    }
    TX_BUFFERS = buf_page as *mut u8;
    for i in 0..NUM_TX_DESCS {
        let desc = &mut *TX_DESCS.add(i);
        desc.addr = 0;
        desc.cmd = 0;
    }
    write_reg(REG_TDBAL, (desc_page as u32) & 0xFFFFFFFF);
    write_reg(REG_TDBAH, 0);
    write_reg(
        REG_TDLEN,
        (NUM_TX_DESCS * core::mem::size_of::<TxDesc>()) as u32,
    );
    write_reg(REG_TDH, 0);
    write_reg(REG_TDT, 0);
    write_reg(REG_TCTL, TCTL_EN | TCTL_PSP | TCTL_CT | TCTL_COLD);
    write_reg(REG_TCTL, TCTL_EN | TCTL_PSP | TCTL_CT | TCTL_COLD);
    write_reg(REG_CTRL, read_reg(REG_CTRL) | CTRL_SLU);
    let status = read_reg(REG_STATUS);
    print(b"[NET] STATUS: 0x");
    crate::vga::print_u64_vga(status as u64);
    if (status & 2) != 0 {
        print(b" (Link Up)\n");
    } else {
        print(b" (Link Down)\n");
    }
    let rctl = read_reg(REG_RCTL);
    print(b"[NET] RCTL: 0x");
    crate::vga::print_u64_vga(rctl as u64);
    print(b"\n");
    print(b"[NET] E1000 Initialized\n");
    RX_CUR = 0;
}
pub unsafe fn poll_packet(buffer: &mut [u8]) -> usize {
    if MMIO_BASE == 0 {
        return 0;
    }
    if RX_DESCS.is_null() {
        return 0;
    }
    let desc_ptr = RX_DESCS.add(RX_CUR);
    let desc_base = desc_ptr as *mut u8;
    let status_ptr = desc_base.add(12);
    let status = core::ptr::read_volatile(status_ptr);
    if (status & 1) != 0 {
        crate::serial::print_serial(b"[NET] Packet Detected! Status: ");
        crate::serial::print_hex(status as u64);
        crate::serial::print_serial(b"\n");
        let len_ptr = desc_base.add(8) as *const u16;
        let len = core::ptr::read_volatile(len_ptr) as usize;
        let addr_ptr = desc_base as *const u64;
        let addr = core::ptr::read_volatile(addr_ptr);
        let ptr = addr as *const u8;
        for i in 0..len {
            if i < buffer.len() {
                buffer[i] = *ptr.add(i);
            }
        }
        core::ptr::write_volatile(status_ptr, 0);
        let old_cur = RX_CUR;
        RX_CUR = (RX_CUR + 1) % NUM_RX_DESCS;
        write_reg(REG_RDT, old_cur as u32);
        return len;
    }
    0
}
pub unsafe fn send_packet(data: &[u8]) {
    if MMIO_BASE == 0 {
        return;
    }
    let tail = read_reg(REG_TDT) as usize;
    let desc = &mut *TX_DESCS.add(tail);
    let buf_addr = TX_BUFFERS.add(tail * 2048);
    for i in 0..data.len() {
        *buf_addr.add(i) = data[i];
    }
    desc.addr = buf_addr as u64;
    desc.length = data.len() as u16;
    desc.cmd = CMD_EOP | CMD_IFCS;
    desc.status = 0;
    let new_tail = (tail + 1) % NUM_TX_DESCS;
    write_reg(REG_TDT, new_tail as u32);
}
unsafe fn write_reg(offset: usize, val: u32) {
    let addr = (MMIO_BASE as usize + offset) as *mut u32;
    *addr = val;
}
unsafe fn read_reg(offset: usize) -> u32 {
    let addr = (MMIO_BASE as usize + offset) as *const u32;
    *addr
}
