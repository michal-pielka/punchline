use std::net::UdpSocket;

use punchline_proto::stun;
use punchline_proto::transport::Transport;
use punchline_proto::udp::UdpTransport;

const ADDRESS: &str = "0.0.0.0";
const PORT: &str = "3478";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sock = UdpTransport::new(UdpSocket::bind(format!("{}:{}", ADDRESS, PORT))?);
    let mut buf = [0u8; 1024];

    println!("STUN server listening on {ADDRESS}:{PORT}");

    loop {
        let (len, src_addr) = sock.recv_from(&mut buf)?;
        let request = &buf[..len];

        let Some(header) = stun::parse_header(request) else {
            eprintln!("Invalid STUN packet from {src_addr}");
            continue;
        };

        if !stun::is_binding_request(&header) {
            eprintln!(
                "Not a Binding Request from {src_addr}, type=0x{:04x}",
                header.msg_type
            );
            continue;
        }

        println!("Binding Request from {src_addr}");

        let Some(response) = stun::build_binding_response(&header.transaction_id, src_addr) else {
            eprintln!("Failed to build response for {src_addr} (IPv6 not supported)");
            continue;
        };

        if let Err(e) = sock.send_to(&response, src_addr) {
            eprintln!("Send error: {e}");
        }
    }
}
