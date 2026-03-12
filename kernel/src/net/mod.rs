use crate::drivers::e1000;
use crate::vga::print;

pub mod arp;

pub mod dns;

pub mod ethernet;

pub mod icmp;

pub mod ipv4;

pub mod udp;
use arp::{ArpPacket, ARP_OP_REPLY, ARP_OP_REQUEST};
use dns::handle_dns_reply;
use ethernet::{htons, EthernetHeader, ETHERTYPE_ARP, ETHERTYPE_IPV4};
use icmp::{IcmpHeader, ICMP_ECHO_REPLY, ICMP_ECHO_REQUEST};
use ipv4::{checksum, Ipv4Header, PROTOCOL_ICMP, PROTOCOL_UDP};
use udp::UdpHeader;
pub const MY_IP: [u8; 4] = [10, 0, 2, 15];

pub fn get_my_mac() -> [u8; 6] {
    unsafe { crate::drivers::e1000::MAC_ADDR }
}
pub const GATEWAY_MAC: [u8; 6] = [0x52, 0x54, 0x00, 0x12, 0x35, 0x02];
pub const GATEWAY_IP: [u8; 4] = [10, 0, 2, 2];
pub static mut PING_REPLY_RECEIVED: bool = false;
pub static mut PING_WAITING: bool = false;
pub static mut PING_EXPECTED_SEQ: u16 = 0;
pub static mut NET_BUFFER: [u8; 2048] = [0; 2048];

pub struct RxPacket {
    pub len: usize,
    pub data: [u8; 2048],
}

pub struct RxQueue {
    pub packets: [RxPacket; 32],
    pub head: usize,
    pub tail: usize,
    pub full: bool,
}

pub static mut RX_QUEUE: RxQueue = RxQueue {
    packets: [RxPacket { len: 0, data: [0; 2048] }; 32],
    head: 0,
    tail: 0,
    full: false,
};

pub fn rx_queue_push(packet: RxPacket) {
    unsafe {
        if RX_QUEUE.full {
            crate::drivers::e1000::DROPPED_PACKETS += 1;
            return;
        }
        RX_QUEUE.packets[RX_QUEUE.tail] = packet;
        RX_QUEUE.tail = (RX_QUEUE.tail + 1) % 32;
        if RX_QUEUE.tail == RX_QUEUE.head {
            RX_QUEUE.full = true;
        }
    }
}

pub fn rx_queue_pop() -> Option<RxPacket> {
    unsafe {
        if RX_QUEUE.head == RX_QUEUE.tail && !RX_QUEUE.full {
            return None;
        }
        let packet = RX_QUEUE.packets[RX_QUEUE.head].clone();
        RX_QUEUE.head = (RX_QUEUE.head + 1) % 32;
        RX_QUEUE.full = false;
        Some(packet)
    }
}

impl Clone for RxPacket {

    fn clone(&self) -> Self {
        *self
    }
}

impl Copy for RxPacket {}

pub extern "C" fn net_task() {
    crate::serial::print_serial(b"[NET] Task Started (Buffered).\n");
    loop {
        unsafe {
            core::arch::asm!("cli");
            if let Some(mut packet) = rx_queue_pop() {
                core::arch::asm!("sti");
                handle_packet(&mut packet.data[..packet.len]);
            } else {
                core::arch::asm!("sti");
                core::arch::asm!("hlt");
            }
        }
    }
}
pub unsafe fn send_echo_request(dest_ip: [u8; 4], seq: u16) {
    let mut packet = [0u8; 64];
    let eth_len = core::mem::size_of::<EthernetHeader>();
    let ip_len = core::mem::size_of::<Ipv4Header>();
    let icmp_len = core::mem::size_of::<IcmpHeader>();
    let eth = &mut *(packet.as_mut_ptr() as *mut EthernetHeader);
    eth.src_mac = get_my_mac();
    eth.dest_mac = GATEWAY_MAC;
    eth.ethertype = htons(ETHERTYPE_IPV4);
    let ip = &mut *(packet.as_mut_ptr().add(eth_len) as *mut Ipv4Header);
    ip.version_ihl = 0x45;
    ip.diff_services = 0;
    ip.total_len = htons((ip_len + icmp_len) as u16);
    ip.id = 0x1234;
    ip.flags_frag_offset = 0;
    ip.ttl = 64;
    ip.protocol = PROTOCOL_ICMP;
    ip.src_ip = MY_IP;
    ip.dest_ip = dest_ip;
    ip.checksum = 0;
    ip.checksum = htons(checksum(core::slice::from_raw_parts(
        ip as *const _ as *const u8,
        ip_len,
    )));
    let icmp = &mut *(packet.as_mut_ptr().add(eth_len + ip_len) as *mut IcmpHeader);
    icmp.packet_type = ICMP_ECHO_REQUEST;
    icmp.code = 0;
    icmp.id = htons(PING_CURRENT_ID);
    icmp.seq = htons(seq);
    icmp.checksum = 0;
    icmp.checksum = htons(checksum(core::slice::from_raw_parts(
        icmp as *const _ as *const u8,
        icmp_len,
    )));
    e1000::send_packet(&packet[..eth_len + ip_len + icmp_len]);
    crate::drivers::e1000::TOTAL_TX_PACKETS += 1;
}
pub unsafe fn handle_packet(packet: &mut [u8]) {
    if packet.len() < core::mem::size_of::<EthernetHeader>() {
        crate::drivers::e1000::DROPPED_PACKETS += 1;
        return;
    }
    let eth_header = &*(packet.as_ptr() as *const EthernetHeader);
    let eth_type = htons(eth_header.ethertype);
    if eth_type == ETHERTYPE_ARP {
        handle_arp(packet);
    } else if eth_type == ETHERTYPE_IPV4 {
        handle_ipv4(packet);
    }
}
unsafe fn handle_arp(packet: &mut [u8]) {
    let eth_len = core::mem::size_of::<EthernetHeader>();
    if packet.len() < eth_len + core::mem::size_of::<ArpPacket>() {
        return;
    }
    let arp_header = &mut *(packet.as_mut_ptr().add(eth_len) as *mut ArpPacket);
    if htons(arp_header.opcode) == ARP_OP_REQUEST {
        if arp_header.dest_ip == MY_IP {
            let target_mac = arp_header.src_mac;
            let target_ip = arp_header.src_ip;
            arp_header.opcode = htons(ARP_OP_REPLY);
            arp_header.src_mac = get_my_mac();
            arp_header.src_ip = MY_IP;
            arp_header.dest_mac = target_mac;
            arp_header.dest_ip = target_ip;
            let eth_header = &mut *(packet.as_mut_ptr() as *mut EthernetHeader);
            eth_header.dest_mac = target_mac;
            eth_header.src_mac = get_my_mac();
            e1000::send_packet(&packet[..eth_len + core::mem::size_of::<ArpPacket>()]);
            crate::drivers::e1000::TOTAL_TX_PACKETS += 1;
        }
    }
}
unsafe fn handle_ipv4(packet: &mut [u8]) {
    let eth_len = core::mem::size_of::<EthernetHeader>();
    if packet.len() < eth_len + core::mem::size_of::<Ipv4Header>() {
        return;
    }
    let ip_header = &mut *(packet.as_mut_ptr().add(eth_len) as *mut Ipv4Header);
    if ip_header.dest_ip != MY_IP {
        return;
    }
    if ip_header.protocol == PROTOCOL_ICMP {
        let ip_header_len = (ip_header.version_ihl & 0x0F) as usize * 4;
        let icmp_offset = eth_len + ip_header_len;
        if packet.len() < icmp_offset + core::mem::size_of::<IcmpHeader>() {
            return;
        }
        let icmp_header = &mut *(packet.as_mut_ptr().add(icmp_offset) as *mut IcmpHeader);
        if icmp_header.packet_type == ICMP_ECHO_REQUEST {
            icmp_header.packet_type = ICMP_ECHO_REPLY;
            icmp_header.checksum = 0;
            let total_len = htons(ip_header.total_len) as usize;
            let icmp_len = total_len - ip_header_len;
            let icmp_data = &mut packet[icmp_offset..icmp_offset + icmp_len];
            icmp_header.checksum = htons(checksum(core::slice::from_raw_parts(
                icmp_data.as_ptr(),
                icmp_len,
            )));
            let src_ip = ip_header.src_ip;
            ip_header.src_ip = MY_IP;
            ip_header.dest_ip = src_ip;
            ip_header.checksum = 0;
            ip_header.checksum = htons(checksum(core::slice::from_raw_parts(
                ip_header as *const _ as *const u8,
                ip_header_len,
            )));
            let eth_header = &mut *(packet.as_mut_ptr() as *mut EthernetHeader);
            let src_mac = eth_header.src_mac;
            eth_header.dest_mac = src_mac;
            eth_header.src_mac = get_my_mac();
            e1000::send_packet(&packet[..eth_len + total_len]);
            crate::drivers::e1000::TOTAL_TX_PACKETS += 1;
        } else if icmp_header.packet_type == ICMP_ECHO_REPLY {
            if PING_WAITING {
                if icmp_header.id == htons(PING_CURRENT_ID) && icmp_header.seq == htons(PING_EXPECTED_SEQ) {
                    crate::serial::print_serial(b"[NET] Ping Reply Received!\n");
                    PING_REPLY_RECEIVED = true;
                } else {
                    crate::serial::print_serial(b"[NET] ICMP Mismatch - Expected ID: ");
                    crate::serial::print_hex(PING_CURRENT_ID as u64);
                    crate::serial::print_serial(b" Seq: ");
                    crate::serial::print_hex(PING_EXPECTED_SEQ as u64);
                    crate::serial::print_serial(b" | Got ID: ");
                    crate::serial::print_hex(htons(icmp_header.id) as u64);
                    crate::serial::print_serial(b" Seq: ");
                    crate::serial::print_hex(htons(icmp_header.seq) as u64);
                    crate::serial::print_serial(b"\n");
                }
            }
        }
    } else if ip_header.protocol == PROTOCOL_UDP {
        let ip_header_len = (ip_header.version_ihl & 0x0F) as usize * 4;
        let udp_offset = eth_len + ip_header_len;
        if packet.len() < udp_offset + core::mem::size_of::<UdpHeader>() {
            return;
        }
        let udp_header = &*(packet.as_ptr().add(udp_offset) as *const UdpHeader);
        let src_port = htons(udp_header.src_port);
        let _dest_port = htons(udp_header.dest_port);
        if src_port == 53 {
            let payload_offset = udp_offset + core::mem::size_of::<UdpHeader>();
            let payload = &packet[payload_offset..];
            handle_dns_reply(payload);
        }
    }
}
pub static mut PING_CURRENT_ID: u16 = 0;

pub fn print_ip(ip: [u8; 4]) {
    for i in 0..4 {
        let mut n = ip[i];
        if n == 0 {
            print(b"0");
        } else {
            let mut buf = [0u8; 3];
            let mut len = 0;
            while n > 0 {
                buf[len] = b'0' + (n % 10);
                n /= 10;
                len += 1;
            }
            for j in 0..(len / 2) {
                let tmp = buf[j];
                buf[j] = buf[len - 1 - j];
                buf[len - 1 - j] = tmp;
            }
            print(&buf[..len]);
        }
        if i < 3 {
            print(b".");
        }
    }
}

pub unsafe fn ping_blocking(dest_ip: [u8; 4]) {
    let session_id = (crate::interrupts::TICKS & 0xFFFF) as u16;
    PING_CURRENT_ID = session_id;

    print(b"PING ");
    print_ip(dest_ip);
    print(b" 32 bytes of data.\n");

    for seq in 1..=4 {
        PING_REPLY_RECEIVED = false;
        PING_WAITING = true;
        PING_EXPECTED_SEQ = seq;
        send_echo_request(dest_ip, seq);
        let mut received = false;
        let start_time = crate::interrupts::TICKS;
        let mut end_time = start_time;
        loop {
            if core::ptr::read_volatile(&raw const PING_REPLY_RECEIVED) {
                received = true;
                end_time = crate::interrupts::TICKS;
                break;
            }
            if crate::interrupts::TICKS - start_time > 2000 {
                break;
            }
            crate::task::schedule();
        }
        PING_WAITING = false;
        if !received {
            print(b"Request timed out.\n");
        } else {
            print(b"Reply received from ");
            print_ip(dest_ip);
            print(b": time=");
            let time_ms = (end_time - start_time) * 10;
            let mut time_buf = [0u8; 20];
            let mut t = time_ms;
            let mut t_len = 0;
            if t == 0 {
                print(b"<1");
            } else {
                while t > 0 {
                    time_buf[t_len] = b'0' + (t % 10) as u8;
                    t /= 10;
                    t_len += 1;
                }
                for j in 0..(t_len / 2) {
                    let tmp = time_buf[j];
                    time_buf[j] = time_buf[t_len - 1 - j];
                    time_buf[t_len - 1 - j] = tmp;
                }
                print(&time_buf[..t_len]);
            }
            print(b"ms\n");

            for _ in 0..50 {
                crate::task::schedule();
            }
        }
    }
}
