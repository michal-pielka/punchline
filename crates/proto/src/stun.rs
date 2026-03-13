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

// A parsed STUN message header.
pub struct StunHeader {
    pub msg_type: u16,
    pub msg_length: u16,
    pub transaction_id: [u8; 12],
}

// Parse a STUN message header from a 20+ byte buffer.
// Returns None if the buffer is too short or the magic cookie doesn't match.
pub fn parse_header(buf: &[u8]) -> Option<StunHeader> {
    if buf.len() < HEADER_SIZE {
        return None;
    }

    let msg_type = u16::from_be_bytes([buf[0], buf[1]]);
    let msg_length = u16::from_be_bytes([buf[2], buf[3]]);
    let cookie = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);

    if cookie != MAGIC_COOKIE {
        return None;
    }

    let mut transaction_id = [0u8; 12];
    transaction_id.copy_from_slice(&buf[8..20]);

    Some(StunHeader {
        msg_type,
        msg_length,
        transaction_id,
    })
}
