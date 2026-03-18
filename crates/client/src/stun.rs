use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;

use punchline_proto::stun;
use punchline_proto::transport::Transport;
use punchline_proto::udp::UdpTransport;
use tracing::debug;

const CLIENT_ADDR: &str = "0.0.0.0:0";
const STUN_TIMEOUT: Duration = Duration::from_secs(3);

pub fn get_external_addr(stun_addr: SocketAddr) -> anyhow::Result<(SocketAddr, UdpTransport)> {
    let sock = UdpTransport::new(UdpSocket::bind(CLIENT_ADDR)?);

    let (request, _transaction_id) = stun::build_binding_request();
    debug!(%stun_addr, "Sending STUN binding request");
    sock.send_to(&request, stun_addr)?;

    let mut buf = [0u8; 1024];
    let (len, _src) = sock.recv_from(&mut buf)?;

    let addr = stun::parse_xor_mapped_address(&buf[..len])?;

    Ok((addr, sock))
}

pub fn test_connection(stun_addr: SocketAddr) -> anyhow::Result<bool> {
    let sock = UdpSocket::bind(CLIENT_ADDR)?;
    sock.set_read_timeout(Some(STUN_TIMEOUT))?;

    let (request, _) = stun::build_binding_request();
    if sock.send_to(&request, stun_addr).is_err() {
        return Ok(false);
    }

    let mut buf = [0u8; 1024];
    let Ok((len, _)) = sock.recv_from(&mut buf) else {
        return Ok(false);
    };

    Ok(stun::parse_xor_mapped_address(&buf[..len]).is_ok())
}
