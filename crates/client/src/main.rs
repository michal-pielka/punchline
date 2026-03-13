use std::net::UdpSocket;

use punchline_proto::stun::{build_binding_request, parse_xor_mapped_address};

const ADDRESS: &str = "0.0.0.0:0";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server_address = std::env::var("SERVER_ADDRESS")?;

    let sock = UdpSocket::bind(ADDRESS)?;
    let _ = sock.connect(server_address);

    let (request, transaction_id) = build_binding_request();
    let _ = sock.send(&request);

    let mut buf = [0u8; 1024];
    let len = sock.recv(&mut buf)?;

    let addr = parse_xor_mapped_address(&buf[..len]);
    println!("Public address: {addr:?}");

    Ok(())
}
