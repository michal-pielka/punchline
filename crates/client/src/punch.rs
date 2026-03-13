use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use punchline_proto::transport::Transport;
use punchline_proto::udp::UdpTransport;
use tracing::{debug, info};

const PUNCH_PROBE: &[u8] = &[0x00];
const PUNCH_ACK: &[u8] = &[0x01];
const PUNCH_TIMEOUT_MS: u64 = 200;

fn send_probe(transport: &UdpTransport, peer_addr: SocketAddr) -> Result<(), std::io::Error> {
    transport.send_to(PUNCH_PROBE, peer_addr)?;
    Ok(())
}

fn send_ack(transport: &UdpTransport, peer_addr: SocketAddr) -> Result<(), std::io::Error> {
    transport.send_to(PUNCH_ACK, peer_addr)?;
    Ok(())
}

fn is_probe(buf: &[u8]) -> bool {
    buf == PUNCH_PROBE
}

fn is_ack(buf: &[u8]) -> bool {
    buf == PUNCH_ACK
}

pub fn establish(
    transport: &UdpTransport,
    peer_addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    let send_transport = transport.try_clone()?;
    let done = Arc::new(AtomicBool::new(false));
    let send_done = done.clone();

    let sender = thread::spawn(move || {
        while !send_done.load(Ordering::Relaxed) {
            if let Err(e) = send_probe(&send_transport, peer_addr) {
                debug!(%e, "Probe send failed");
            }
            thread::sleep(Duration::from_millis(PUNCH_TIMEOUT_MS));
        }
    });

    let mut buf = [0u8; 1024];
    loop {
        let (len, src_addr) = transport.recv_from(&mut buf)?;
        let data = &buf[..len];

        if src_addr != peer_addr {
            continue;
        }

        if is_probe(data) {
            debug!("Received probe, sending ACK");
            send_ack(transport, peer_addr)?;
        } else if is_ack(data) {
            info!("Hole punched!");
            done.store(true, Ordering::Relaxed);
            break;
        }
    }

    sender.join().map_err(|_| "Sender thread panicked")?;

    Ok(())
}
