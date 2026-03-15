use std::net::UdpSocket;

use anyhow::Context;
use clap::Parser;
use punchline_proto::stun;
use punchline_proto::transport::Transport;
use punchline_proto::udp::UdpTransport;
use punchline_stund::cli::Args;
use tracing::{debug, error, info, warn};

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let log_level = if args.quiet {
        None
    } else {
        match args.verbose {
            0 => Some(tracing::Level::INFO),
            1 => Some(tracing::Level::DEBUG),
            _ => Some(tracing::Level::TRACE),
        }
    };

    if let Some(level) = log_level {
        tracing_subscriber::fmt().with_max_level(level).init();
    }

    let sock = UdpTransport::new(
        UdpSocket::bind(format!("{}:{}", args.address, args.port))
            .context("Failed to bind STUN server socket")?,
    );
    let mut buf = [0u8; 1024];

    info!("STUN server listening on {}:{}", args.address, args.port);

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
