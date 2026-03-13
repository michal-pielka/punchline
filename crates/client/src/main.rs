use std::net::UdpSocket;

use punchline_proto::stun::{build_binding_request, parse_xor_mapped_address};
use punchline_proto::transport::Transport;
use punchline_proto::udp::UdpTransport;

const STUN_ADDRESS: &str = "0.0.0.0:0";
const SIGNAL_ADDRESS: &str = "0.0.0.0:0";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stun_address: std::net::SocketAddr = std::env::var("STUN_ADDRESS")?.parse()?;
    let signal_address: std::net::SocketAddr = std::env::var("SIGNAL_ADDRESS")?.parse()?;

    let sock = UdpTransport::new(UdpSocket::bind(STUN_ADDRESS)?);

    let (request, _transaction_id) = build_binding_request();
    sock.send_to(&request, stun_address)?;

    let mut buf = [0u8; 1024];
    let (len, _src) = sock.recv_from(&mut buf)?;

    let addr = parse_xor_mapped_address(&buf[..len]);
    println!("Public address: {addr:?}");

    Ok(())
}
