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

// Returns true if the message type is a Binding Request.
pub fn is_binding_request(header: &StunHeader) -> bool {
    header.msg_type == BINDING_REQUEST
}

// Build a complete Binding Response with XOR-MAPPED-ADDRESS for the given source address.
// Only supports IPv4.
pub fn build_binding_response(transaction_id: &[u8; 12], src_addr: SocketAddr) -> Option<Vec<u8>> {
    let (ip_bytes, port) = match src_addr {
        SocketAddr::V4(addr) => (addr.ip().octets(), addr.port()),
        SocketAddr::V6(_) => return None, // IPv6 not supported yet
    };

    // XOR the port with the top 16 bits of the magic cookie
    let xor_port = port ^ (MAGIC_COOKIE >> 16) as u16;

    // XOR the IP with the full magic cookie
    let cookie_bytes = MAGIC_COOKIE.to_be_bytes();
    let xor_ip = [
        ip_bytes[0] ^ cookie_bytes[0],
        ip_bytes[1] ^ cookie_bytes[1],
        ip_bytes[2] ^ cookie_bytes[2],
        ip_bytes[3] ^ cookie_bytes[3],
    ];

    // XOR-MAPPED-ADDRESS attribute: 4 bytes attr header + 8 bytes value = 12 bytes
    let attr_value_len: u16 = 8;
    let attr_total_len: u16 = 4 + attr_value_len; // type(2) + length(2) + value(8)

    // Total message: 20 byte header + 12 byte attribute
    let mut resp = Vec::with_capacity(HEADER_SIZE + attr_total_len as usize);

    // Header
    resp.extend_from_slice(&BINDING_RESPONSE.to_be_bytes());
    resp.extend_from_slice(&attr_total_len.to_be_bytes()); // message length = bytes after header
    resp.extend_from_slice(&MAGIC_COOKIE.to_be_bytes());
    resp.extend_from_slice(transaction_id);

    // XOR-MAPPED-ADDRESS attribute
    resp.extend_from_slice(&ATTR_XOR_MAPPED_ADDRESS.to_be_bytes());
    resp.extend_from_slice(&attr_value_len.to_be_bytes());
    resp.push(0x00); // reserved
    resp.push(FAMILY_IPV4);
    resp.extend_from_slice(&xor_port.to_be_bytes());
    resp.extend_from_slice(&xor_ip);

    Some(resp)
}
