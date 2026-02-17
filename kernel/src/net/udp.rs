use super::ipv4::checksum;
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct UdpHeader {
    pub src_port: u16,
    pub dest_port: u16,
    pub length: u16,
    pub checksum: u16,
}
pub fn udp_checksum(
    src_ip: &[u8; 4],
    dest_ip: &[u8; 4],
    payload: &[u8],
    src_port: u16,
    dest_port: u16,
) -> u16 {
    let udp_len = (8 + payload.len()) as u16;
    let mut sum: u32 = 0;
    for i in 0..2 {
        let word = ((src_ip[i * 2] as u16) << 8) | (src_ip[i * 2 + 1] as u16);
        sum += word as u32;
    }
    for i in 0..2 {
        let word = ((dest_ip[i * 2] as u16) << 8) | (dest_ip[i * 2 + 1] as u16);
        sum += word as u32;
    }
    sum += 0x0011;
    sum += udp_len as u32;
    sum += src_port as u32;
    sum += dest_port as u32;
    sum += udp_len as u32;
    let mut chunks = payload.chunks_exact(2);
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
