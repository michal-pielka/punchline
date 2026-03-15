use std::net::UdpSocket;

use anyhow::Context;
use punchline_proto::stun;
use punchline_proto::transport::Transport;
use punchline_proto::udp::UdpTransport;
use tracing::{debug, error, info, warn};

const ADDRESS: &str = "0.0.0.0";
const PORT: &str = "3478";

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let sock = UdpTransport::new(
        UdpSocket::bind(format!("{}:{}", ADDRESS, PORT))
            .context("Failed to bind STUN server socket")?,
    );
    let mut buf = [0u8; 1024];

    info!("STUN server listening on {ADDRESS}:{PORT}");

    loop {
        let (len, src_addr) = sock.recv_from(&mut buf)?;
        let request = &buf[..len];

        let header = match stun::parse_header(request) {
            Ok(h) => h,
            Err(e) => {
                warn!(%src_addr, %e, "Invalid STUN packet");
                continue;
            }
        };

        if !stun::is_binding_request(&header) {
            warn!(%src_addr, msg_type = format_args!("0x{:04x}", header.msg_type), "Unexpected message type");
            continue;
        }

        debug!(%src_addr, "Binding Request");

        let response = match stun::build_binding_response(&header.transaction_id, src_addr) {
            Ok(r) => r,
            Err(e) => {
                warn!(%src_addr, %e, "Failed to build response");
                continue;
            }
        };

        if let Err(e) = sock.send_to(&response, src_addr) {
            error!(%src_addr, %e, "Failed to send response");
        }
    }
}
