use std::net::{SocketAddr, UdpSocket};

use punchline_proto::stun;
use punchline_proto::transport::Transport;
use punchline_proto::udp::UdpTransport;
use tracing::debug;

const CLIENT_ADDR: &str = "0.0.0.0:0";

pub fn get_external_addr(
    stun_addr: SocketAddr,
) -> Result<(SocketAddr, UdpTransport), Box<dyn std::error::Error>> {
    let sock = UdpTransport::new(UdpSocket::bind(CLIENT_ADDR)?);

    let (request, _transaction_id) = stun::build_binding_request();
    debug!(%stun_addr, "Sending STUN binding request");
    sock.send_to(&request, stun_addr)?;

    let mut buf = [0u8; 1024];
    let (len, _src) = sock.recv_from(&mut buf)?;

    let addr = stun::parse_xor_mapped_address(&buf[..len]).ok_or("Failed to parse address")?;

    Ok((addr, sock))
}
