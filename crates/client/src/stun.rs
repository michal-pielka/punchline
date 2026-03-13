use std::net::{SocketAddr, UdpSocket};

use punchline_proto::stun::{build_binding_request, parse_xor_mapped_address};
use punchline_proto::transport::Transport;
use punchline_proto::udp::UdpTransport;

const CLIENT_ADDRESS: &str = "0.0.0.0:0";

pub fn get_external_address(
    stun_address: SocketAddr,
) -> Result<SocketAddr, Box<dyn std::error::Error>> {
    let sock = UdpTransport::new(UdpSocket::bind(CLIENT_ADDRESS)?);

    let (request, _transaction_id) = build_binding_request();
    sock.send_to(&request, stun_address)?;

    let mut buf = [0u8; 1024];
    let (len, _src) = sock.recv_from(&mut buf)?;

    let addr = parse_xor_mapped_address(&buf[..len]).ok_or("Failed to parse address")?;

    Ok(addr)
}
