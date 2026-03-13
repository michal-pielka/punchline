use std::io;
use std::net::SocketAddr;
use std::thread;

use punchline_proto::transport::Transport;
use punchline_proto::udp::UdpTransport;
use tracing::{debug, error, info};

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

fn recv_loop(
    transport: &UdpTransport,
    peer_addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = [0u8; 1024];

    loop {
        let (len, src_addr) = transport.recv_from(&mut buf)?;

        if src_addr != peer_addr {
            continue;
        }

        if len == 0 || buf[0] != MSG_PREFIX {
            continue;
        }

        let msg = String::from_utf8_lossy(&buf[1..len]);
        info!("{msg}");
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

    recv_loop(transport, peer_addr)?;

    Ok(())
}
