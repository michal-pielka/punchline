use std::net::SocketAddr;

const MAGIC_COOKIE: u32 = 0x2112_A442;
const HEADER_SIZE: usize = 20;

// Message types
const BINDING_REQUEST: u16 = 0x0001;
const BINDING_RESPONSE: u16 = 0x0101;

// Attribute types
const ATTR_XOR_MAPPED_ADDRESS: u16 = 0x0020;

// Address families
const FAMILY_IPV4: u8 = 0x01;

/// A parsed STUN message header.
pub struct StunHeader {
    pub msg_type: u16,
    pub msg_length: u16,
    pub transaction_id: [u8; 12],
}
