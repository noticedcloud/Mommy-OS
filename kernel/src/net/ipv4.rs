#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Ipv4Header {
    pub version_ihl: u8,
    pub diff_services: u8,
    pub total_len: u16,
    pub id: u16,
    pub flags_frag_offset: u16,
    pub ttl: u8,
    pub protocol: u8,
    pub checksum: u16,
    pub src_ip: [u8; 4],
    pub dest_ip: [u8; 4],
}
pub const PROTOCOL_ICMP: u8 = 1;
pub const PROTOCOL_TCP: u8 = 6;
pub const PROTOCOL_UDP: u8 = 17;
pub fn checksum(data: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    let mut chunks = data.chunks_exact(2);
    for chunk in chunks.by_ref() {
        let word = ((chunk[0] as u16) << 8) | (chunk[1] as u16);
        sum = sum.wrapping_add(word as u32);
    }
    if let Some(&last) = chunks.remainder().get(0) {
        let word = (last as u16) << 8;
        sum = sum.wrapping_add(word as u32);
    }
    while (sum >> 16) != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !sum as u16
}
