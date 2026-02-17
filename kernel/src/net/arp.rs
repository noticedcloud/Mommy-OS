use super::ethernet::htons;
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct ArpPacket {
    pub hardware_type: u16,
    pub protocol_type: u16,
    pub hardware_len: u8,
    pub protocol_len: u8,
    pub opcode: u16,
    pub src_mac: [u8; 6],
    pub src_ip: [u8; 4],
    pub dest_mac: [u8; 6],
    pub dest_ip: [u8; 4],
}
pub const ARP_OP_REQUEST: u16 = 1;
pub const ARP_OP_REPLY: u16 = 2;
pub const HARDWARE_TYPE_ETHERNET: u16 = 1;
