use std::net::SocketAddr;

use rand_core::RngCore;

use crate::error::ProtoError;

const MAGIC_COOKIE: u32 = 0x2112_A442;
const HEADER_SIZE: usize = 20;

// Message types
const BINDING_REQUEST: u16 = 0x0001;
const BINDING_RESPONSE: u16 = 0x0101;

// Attribute types
const ATTR_XOR_MAPPED_ADDRESS: u16 = 0x0020;

// Address families
const FAMILY_IPV4: u8 = 0x01;

pub struct StunHeader {
    pub msg_type: u16,
    pub msg_length: u16,
    pub transaction_id: [u8; 12],
}

pub fn parse_header(buf: &[u8]) -> Result<StunHeader, ProtoError> {
    if buf.len() < HEADER_SIZE {
        return Err(ProtoError::StunBufferTooShort);
    }

    let msg_type = u16::from_be_bytes([buf[0], buf[1]]);
    let msg_length = u16::from_be_bytes([buf[2], buf[3]]);
    let cookie = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);

    if cookie != MAGIC_COOKIE {
        return Err(ProtoError::StunInvalidCookie);
    }

    let mut transaction_id = [0u8; 12];
    transaction_id.copy_from_slice(&buf[8..20]);

    Ok(StunHeader {
        msg_type,
        msg_length,
        transaction_id,
    })
}

pub fn is_binding_request(header: &StunHeader) -> bool {
    header.msg_type == BINDING_REQUEST
}

pub fn build_binding_response(
    transaction_id: &[u8; 12],
    src_addr: SocketAddr,
) -> Result<Vec<u8>, ProtoError> {
    let (ip_bytes, port) = match src_addr {
        SocketAddr::V4(addr) => (addr.ip().octets(), addr.port()),
        SocketAddr::V6(_) => return Err(ProtoError::StunIpv6Unsupported),
    };

    let xor_port = port ^ (MAGIC_COOKIE >> 16) as u16;

    let cookie_bytes = MAGIC_COOKIE.to_be_bytes();
    let xor_ip = [
        ip_bytes[0] ^ cookie_bytes[0],
        ip_bytes[1] ^ cookie_bytes[1],
        ip_bytes[2] ^ cookie_bytes[2],
        ip_bytes[3] ^ cookie_bytes[3],
    ];

    let attr_value_len: u16 = 8;
    let attr_total_len: u16 = 4 + attr_value_len;

    let mut resp = Vec::with_capacity(HEADER_SIZE + attr_total_len as usize);

    // Header
    resp.extend_from_slice(&BINDING_RESPONSE.to_be_bytes());
    resp.extend_from_slice(&attr_total_len.to_be_bytes());
    resp.extend_from_slice(&MAGIC_COOKIE.to_be_bytes());
    resp.extend_from_slice(transaction_id);

    // XOR-MAPPED-ADDRESS attribute
    resp.extend_from_slice(&ATTR_XOR_MAPPED_ADDRESS.to_be_bytes());
    resp.extend_from_slice(&attr_value_len.to_be_bytes());
    resp.push(0x00);
    resp.push(FAMILY_IPV4);
    resp.extend_from_slice(&xor_port.to_be_bytes());
    resp.extend_from_slice(&xor_ip);

    Ok(resp)
}

pub fn parse_xor_mapped_address(buf: &[u8]) -> Result<SocketAddr, ProtoError> {
    let header = parse_header(buf)?;

    if header.msg_type != BINDING_RESPONSE {
        return Err(ProtoError::StunNotBindingResponse);
    }

    let mut pos = HEADER_SIZE;
    while pos + 4 <= buf.len() {
        let attr_type = u16::from_be_bytes([buf[pos], buf[pos + 1]]);
        let attr_len = u16::from_be_bytes([buf[pos + 2], buf[pos + 3]]) as usize;

        if pos + 4 + attr_len > buf.len() {
            return Err(ProtoError::StunInvalidAddress);
        }

        if attr_type == ATTR_XOR_MAPPED_ADDRESS {
            let value = &buf[pos + 4..pos + 4 + attr_len];
            if value.len() < 8 || value[1] != FAMILY_IPV4 {
                return Err(ProtoError::StunInvalidAddress);
            }

            let xor_port = u16::from_be_bytes([value[2], value[3]]);
            let port = xor_port ^ (MAGIC_COOKIE >> 16) as u16;

            let cookie_bytes = MAGIC_COOKIE.to_be_bytes();
            let ip = std::net::Ipv4Addr::new(
                value[4] ^ cookie_bytes[0],
                value[5] ^ cookie_bytes[1],
                value[6] ^ cookie_bytes[2],
                value[7] ^ cookie_bytes[3],
            );

            return Ok(SocketAddr::new(ip.into(), port));
        }

        pos += 4 + ((attr_len + 3) & !3);
    }

    Err(ProtoError::StunMissingAddress)
}

pub fn build_binding_request() -> (Vec<u8>, [u8; 12]) {
    let mut transaction_id = [0u8; 12];
    rand_core::OsRng.fill_bytes(&mut transaction_id);

    let mut req = Vec::with_capacity(HEADER_SIZE);
    req.extend_from_slice(&BINDING_REQUEST.to_be_bytes());
    req.extend_from_slice(&0u16.to_be_bytes());
    req.extend_from_slice(&MAGIC_COOKIE.to_be_bytes());
    req.extend_from_slice(&transaction_id);

    (req, transaction_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binding_response_round_trip() {
        let transaction_id = [0u8; 12];
        let src_addr: SocketAddr = "192.168.0.69:42069".parse().unwrap();
        let response = build_binding_response(&transaction_id, src_addr).unwrap();
        let parsed = parse_xor_mapped_address(&response).unwrap();
        assert_eq!(src_addr, parsed);
    }

    #[test]
    fn parse_header_too_short() {
        assert!(matches!(
            parse_header(&[0u8; 5]),
            Err(ProtoError::StunBufferTooShort)
        ));
    }

    #[test]
    fn parse_header_bad_cookie() {
        let mut buf = [0u8; 20];
        buf[4..8].copy_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);
        assert!(matches!(
            parse_header(&buf),
            Err(ProtoError::StunInvalidCookie)
        ));
    }

    #[test]
    fn build_binding_response_rejects_ipv6() {
        let addr: SocketAddr = "[::1]:1234".parse().unwrap();
        let txn_id = [0u8; 12];
        assert!(matches!(
            build_binding_response(&txn_id, addr),
            Err(ProtoError::StunIpv6Unsupported)
        ));
    }
}
