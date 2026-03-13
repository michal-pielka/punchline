use std::io;
use std::net::SocketAddr;
use std::thread;

use punchline_proto::transport::Transport;
use punchline_proto::udp::UdpTransport;
use tracing::{debug, error};

const MSG_PREFIX: u8 = 0x02;

fn send_loop(
    transport: UdpTransport,
    peer_addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    let mut line = String::new();

    loop {
        line.clear();
        stdin.read_line(&mut line)?;

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let mut payload = vec![MSG_PREFIX];
        payload.extend_from_slice(trimmed.as_bytes());

        transport.send_to(&payload, peer_addr)?;
        debug!("Sent: {trimmed}");
    }
}

pub fn start(
    transport: &UdpTransport,
    peer_addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    let send_transport = transport.try_clone()?;

    thread::spawn(move || {
        if let Err(e) = send_loop(send_transport, peer_addr) {
            error!(%e, "Send loop error");
        }
    });

    Ok(())
}
