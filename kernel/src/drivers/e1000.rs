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
#[allow(dead_code)]
const RCTL_LPE: u32 = 1 << 5;
const RCTL_BAM: u32 = 1 << 15;
const TCTL_EN: u32 = 1 << 1;
const TCTL_PSP: u32 = 1 << 3;
const TCTL_CT: u32 = 0x0F << 4;
const TCTL_COLD: u32 = 0x40 << 12;
const CMD_EOP: u8 = 1 << 0;
const CMD_IFCS: u8 = 1 << 1;
const CMD_RS: u8 = 1 << 3;
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

static mut RX_DESCS: *mut RxDesc = 0 as *mut RxDesc;
static mut RX_BUFFER_ADDRS: [u64; 32] = [0; 32];
static mut RX_CUR: usize = 0;
const NUM_TX_DESCS: usize = 8;
const NUM_RX_DESCS: usize = 32;
const TX_BUF_PAGES: usize = (NUM_TX_DESCS * 2048 + 4095) / 4096;
pub static mut TOTAL_RX_PACKETS: usize = 0;
pub static mut TOTAL_TX_PACKETS: usize = 0;
pub static mut DROPPED_PACKETS: usize = 0;
pub static mut MAC_ADDR: [u8; 6] = [0; 6];
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

    let irq_line = (pci_read(bus_found as u8, slot_found as u8, 0, 0x3C) & 0xFF) as u8;
    print(b"[NET] E1000 Found at ");
    crate::vga::print_u64_vga(bus_found as u64);
    print(b":");
    crate::vga::print_u64_vga(slot_found as u64);
    print(b" IRQ: ");
    crate::vga::print_u64_vga(irq_line as u64);
    print(b"\n");

    let idt_vec = 0x20 + irq_line as usize;
    crate::idt::set_idt_gate(idt_vec, crate::interrupts::e1000_handler_stub as *const () as u64, 0);
    crate::idt::pic_unmask(irq_line);
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
    write_reg(REG_IMC, 0xFFFFFFFF);

    let mut mac_addr = [0u8; 6];
    let temp = read_eeprom(0);
    mac_addr[0] = (temp & 0xFF) as u8;
    mac_addr[1] = (temp >> 8) as u8;
    let temp = read_eeprom(1);
    mac_addr[2] = (temp & 0xFF) as u8;
    mac_addr[3] = (temp >> 8) as u8;
    let temp = read_eeprom(2);
    mac_addr[4] = (temp & 0xFF) as u8;
    mac_addr[5] = (temp >> 8) as u8;

    MAC_ADDR = mac_addr;

    print(b"[NET] MAC Address: ");
    for i in 0..6 {
        crate::serial::print_hex(mac_addr[i] as u64);
        if i < 5 { print(b":"); }
    }
    print(b"\n");

    let mac_low: u32 = mac_addr[0] as u32 | ((mac_addr[1] as u32) << 8) | ((mac_addr[2] as u32) << 16) | ((mac_addr[3] as u32) << 24);
    let mac_high: u32 = mac_addr[4] as u32 | ((mac_addr[5] as u32) << 8) | 0x80000000;
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
        RX_BUFFER_ADDRS[i] = buf_addr;
        desc.addr = buf_addr as u64;
        desc.length = 0;
        desc.checksum = 0;
        desc.status = 0;
        desc.errors = 0;
        desc.special = 0;
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
    let tx_buf_base = crate::pmm::allocate_contiguous_frames(TX_BUF_PAGES);
    if tx_buf_base.is_none() {
        print(b"[NET] FATAL: Cannot allocate contiguous TX buffers!\n");
        return;
    }
    TX_BUFFERS = tx_buf_base.unwrap() as *mut u8;
    print(b"[NET] TX Buffers (contiguous): ");
    crate::vga::print_u64_vga(TX_BUFFERS as u64);
    print(b"\n");
    for i in 0..NUM_TX_DESCS {
        let desc = &mut *TX_DESCS.add(i);
        desc.addr = 0;
        desc.cmd = 0;
        desc.length = 0;
        desc.cso = 0;
        desc.status = 0;
        desc.css = 0;
        desc.special = 0;
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

    write_reg(REG_IMS, 0x1F6DC);

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
        TOTAL_RX_PACKETS += 1;
        return len;
    }
    0
}
pub unsafe fn send_packet(data: &[u8]) {
    if MMIO_BASE == 0 {
        return;
    }
    let saved_flags: u64;
    core::arch::asm!("pushfq; pop {}; cli", out(reg) saved_flags);

    let mut sent = 0;
    while sent < data.len() {
        let tail = read_reg(REG_TDT) as usize;
        let desc = &mut *TX_DESCS.add(tail);

        if desc.cmd != 0 {
             let mut timeout = 0;
             while (core::ptr::read_volatile(&desc.status) & 1) == 0 {
                 timeout += 1;
                 if timeout > 100000 {
                     DROPPED_PACKETS += 1;
                     core::arch::asm!("push {}; popfq", in(reg) saved_flags);
                     return;
                 }
                 core::arch::asm!("pause");
             }
        }

        let buf_addr = TX_BUFFERS.add(tail * 2048);

        let remaining = data.len() - sent;
        let chunk_len = if remaining > 2048 { 2048 } else { remaining };

        for i in 0..chunk_len {
            *buf_addr.add(i) = data[sent + i];
        }

        desc.addr = buf_addr as u64;
        desc.length = chunk_len as u16;

        if sent + chunk_len >= data.len() {
            desc.cmd = CMD_EOP | CMD_IFCS | CMD_RS;
        } else {
            desc.cmd = 0;
        }

        desc.status = 0;

        sent += chunk_len;
        let new_tail = (tail + 1) % NUM_TX_DESCS;
        write_reg(REG_TDT, new_tail as u32);
    }
    TOTAL_TX_PACKETS += 1;
    core::arch::asm!("push {}; popfq", in(reg) saved_flags);
}
unsafe fn write_reg(offset: usize, val: u32) {
    let addr = (MMIO_BASE as usize + offset) as *mut u32;
    core::ptr::write_volatile(addr, val);
}
unsafe fn read_reg(offset: usize) -> u32 {
    let addr = (MMIO_BASE as usize + offset) as *const u32;
    core::ptr::read_volatile(addr)
}
pub fn get_stats() -> (usize, usize, usize) {
    unsafe {
        (TOTAL_RX_PACKETS, TOTAL_TX_PACKETS, DROPPED_PACKETS)
    }
}

unsafe fn read_eeprom(addr: u8) -> u16 {
    let mut tmp: u32 = 0;
    write_reg(REG_EEPROM, 1 | ((addr as u32) << 8));
    let mut timeout = 0;
    while (tmp & (1 << 4)) == 0 && timeout < 100000 {
        tmp = read_reg(REG_EEPROM);
        timeout += 1;
    }
    if timeout >= 100000 {
        crate::serial::print_serial(b"[NET] WARNING: EEPROM read timeout for addr ");
        crate::serial::print_hex(addr as u64);
        crate::serial::print_serial(b"\n");
    }
    ((tmp >> 16) & 0xFFFF) as u16
}

pub unsafe fn handle_interrupt() {
    let status = read_reg(REG_ICR);
    if (status & 0x01) != 0 {
    }
    if (status & 0x80) != 0 {
        loop {
            let mut packet = crate::net::RxPacket { len: 0, data: [0; 2048] };
            let len = poll_packet(&mut packet.data);
            if len > 0 {
                 packet.len = len;
                 crate::net::rx_queue_push(packet);
            } else {
                break;
            }
        }
    }
}
