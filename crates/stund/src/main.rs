use std::net::UdpSocket;

use punchline_proto::stun;
use punchline_proto::transport::Transport;
use punchline_proto::udp::UdpTransport;
use tracing::{debug, error, info, warn};

const ADDRESS: &str = "0.0.0.0";
const PORT: &str = "3478";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let sock = UdpTransport::new(UdpSocket::bind(format!("{}:{}", ADDRESS, PORT))?);
    let mut buf = [0u8; 1024];

    info!("STUN server listening on {ADDRESS}:{PORT}");

    loop {
        let (len, src_addr) = sock.recv_from(&mut buf)?;
        let request = &buf[..len];

        let Some(header) = stun::parse_header(request) else {
            warn!(%src_addr, "Invalid STUN packet");
            continue;
        };

        if !stun::is_binding_request(&header) {
            warn!(%src_addr, msg_type = format_args!("0x{:04x}", header.msg_type), "Unexpected message type");
            continue;
        }

        debug!(%src_addr, "Binding Request");

        let Some(response) = stun::build_binding_response(&header.transaction_id, src_addr) else {
            warn!(%src_addr, "Failed to build response (IPv6 not supported)");
            continue;
        };

        if let Err(e) = sock.send_to(&response, src_addr) {
            error!(%src_addr, %e, "Failed to send response");
        }
    }
}
