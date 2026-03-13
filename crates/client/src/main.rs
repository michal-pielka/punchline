use std::net::UdpSocket;

use punchline_proto::stun::{build_binding_request, parse_xor_mapped_address};
use punchline_proto::transport::Transport;
use punchline_proto::udp::UdpTransport;

const ADDRESS: &str = "0.0.0.0:0";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server_address: std::net::SocketAddr = std::env::var("SERVER_ADDRESS")?.parse()?;

    let sock = UdpTransport::new(UdpSocket::bind(ADDRESS)?);

    let (request, _transaction_id) = build_binding_request();
    sock.send_to(&request, server_address)?;

    let mut buf = [0u8; 1024];
    let (len, _src) = sock.recv_from(&mut buf)?;

    let addr = parse_xor_mapped_address(&buf[..len]);
    println!("Public address: {addr:?}");

    Ok(())
}
