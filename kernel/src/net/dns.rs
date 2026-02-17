use super::ethernet::{htons, EthernetHeader, ETHERTYPE_IPV4};
use super::ipv4::{checksum, Ipv4Header, PROTOCOL_UDP};
use super::udp::{udp_checksum, UdpHeader};
use super::{GATEWAY_MAC, MY_IP, MY_MAC};
use crate::drivers::e1000::send_packet;
use crate::vga::print;
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Default)]
struct DnsHeader {
    id: u16,
    flags: u16,
    qdcount: u16,
    ancount: u16,
    nscount: u16,
    arcount: u16,
}
pub static mut RESOLVED_IP: Option<[u8; 4]> = None;
pub static mut DNS_WAITING: bool = false;
pub unsafe fn resolve_hostname(hostname: &str) -> Option<[u8; 4]> {
    RESOLVED_IP = None;
    DNS_WAITING = true;
    send_dns_query(hostname);
    let mut _buffer = [0u8; 2048];
    let start_time = crate::interrupts::TICKS;
    loop {
        if let Some(ip) = RESOLVED_IP {
            DNS_WAITING = false;
            return Some(ip);
        }
        if crate::interrupts::TICKS - start_time > 500 {
            DNS_WAITING = false;
            return None;
        }
        crate::task::schedule();
    }
    DNS_WAITING = false;
    None
}
unsafe fn send_dns_query(hostname: &str) {
    let mut packet = [0u8; 512];
    let eth_len = core::mem::size_of::<EthernetHeader>();
    let ip_len = core::mem::size_of::<Ipv4Header>();
    let udp_len = core::mem::size_of::<UdpHeader>();
    let dns_header_len = core::mem::size_of::<DnsHeader>();
    let mut offset = eth_len + ip_len + udp_len + dns_header_len;
    let qname_start = eth_len + ip_len + udp_len + dns_header_len;
    let mut q_ptr = qname_start;
    for part in hostname.split('.') {
        let len = part.len();
        packet[q_ptr] = len as u8;
        q_ptr += 1;
        for b in part.as_bytes() {
            packet[q_ptr] = *b;
            q_ptr += 1;
        }
    }
    packet[q_ptr] = 0;
    q_ptr += 1;
    packet[q_ptr] = 0;
    q_ptr += 1;
    packet[q_ptr] = 1;
    q_ptr += 1;
    packet[q_ptr] = 0;
    q_ptr += 1;
    packet[q_ptr] = 1;
    q_ptr += 1;
    let dns_payload_len = q_ptr - (eth_len + ip_len + udp_len);
    let total_len = q_ptr;
    let dns = &mut *(packet.as_mut_ptr().add(eth_len + ip_len + udp_len) as *mut DnsHeader);
    dns.id = htons(0x1337);
    dns.flags = htons(0x0100);
    dns.qdcount = htons(1);
    let udp = &mut *(packet.as_mut_ptr().add(eth_len + ip_len) as *mut UdpHeader);
    udp.src_port = htons(53);
    udp.src_port = htons(50000);
    udp.dest_port = htons(53);
    let udp_total_len = (8 + dns_payload_len) as u16;
    udp.length = htons(udp_total_len);
    udp.checksum = 0;
    let ip = &mut *(packet.as_mut_ptr().add(eth_len) as *mut Ipv4Header);
    ip.version_ihl = 0x45;
    ip.total_len = htons((20 + udp_total_len) as u16);
    ip.id = 0x1234;
    ip.ttl = 64;
    ip.protocol = PROTOCOL_UDP;
    ip.src_ip = MY_IP;
    ip.dest_ip = [8, 8, 8, 8];
    ip.checksum = 0;
    ip.checksum = htons(checksum(core::slice::from_raw_parts(
        ip as *const _ as *const u8,
        20,
    )));
    let eth = &mut *(packet.as_mut_ptr() as *mut EthernetHeader);
    eth.src_mac = MY_MAC;
    eth.dest_mac = GATEWAY_MAC;
    eth.ethertype = htons(ETHERTYPE_IPV4);
    send_packet(&packet[..total_len]);
}
pub unsafe fn handle_dns_reply(data: &[u8]) {
    if data.len() < 12 {
        return;
    }
    let mut idx = 12;
    while idx < data.len() {
        let len = data[idx];
        if len == 0 {
            idx += 1;
            break;
        }
        idx += len as usize + 1;
    }
    idx += 4;
    if idx >= data.len() {
        return;
    }
    while idx < data.len() {
        if (data[idx] & 0xC0) == 0xC0 {
            idx += 2;
        } else {
            while idx < data.len() && data[idx] != 0 {
                idx += data[idx] as usize + 1;
            }
            idx += 1;
        }
        if idx + 10 > data.len() {
            return;
        }
        let type_code: u16 = ((data[idx] as u16) << 8) | (data[idx + 1] as u16);
        let _class_code: u16 = ((data[idx + 2] as u16) << 8) | (data[idx + 3] as u16);
        let _ttl: u32 = 0;
        let rd_len: u16 = ((data[idx + 8] as u16) << 8) | (data[idx + 9] as u16);
        idx += 10;
        if type_code == 1 && rd_len == 4 {
            if idx + 4 <= data.len() {
                let ip = [data[idx], data[idx + 1], data[idx + 2], data[idx + 3]];
                print(b"[DNS] IP Solved: ");
                crate::vga::print_u64_vga(ip[0] as u64);
                print(b".");
                crate::vga::print_u64_vga(ip[1] as u64);
                print(b".");
                crate::vga::print_u64_vga(ip[2] as u64);
                print(b".");
                crate::vga::print_u64_vga(ip[3] as u64);
                print(b"\n");
                RESOLVED_IP = Some(ip);
                return;
            }
        }
        idx += rd_len as usize;
    }
}
