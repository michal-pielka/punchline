use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use punchline_proto::transport::Transport;
use punchline_proto::udp::UdpTransport;
use tracing::{debug, info};

const PUNCH_PROBE: &[u8] = &[0x00];
const PUNCH_ACK: &[u8] = &[0x01];
const PUNCH_INTERVAL_MS: u64 = 200;
const ACK_TIMEOUT: Duration = Duration::from_secs(2);

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

/// Establish a UDP hole punch with the peer.
///
/// Protocol:
///   1. Both sides send PROBE packets until they receive one.
///   2. Upon receiving a PROBE, send ACK and keep listening.
///   3. Upon receiving an ACK, send ACK back (so peer also finishes) and exit.
///
/// Both sides exit once they have sent AND received an ACK.
pub fn establish(
    transport: &UdpTransport,
    peer_addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    let send_transport = transport.try_clone()?;
    let done = Arc::new(AtomicBool::new(false));
    let send_done = done.clone();
    let got_probe = Arc::new(AtomicBool::new(false));
    let sender_got_probe = got_probe.clone();

    // Sender thread: sends PROBEs until we've received a probe,
    // then switches to sending ACKs until done or timeout.
    let sender = thread::spawn(move || {
        let mut ack_mode_since: Option<Instant> = None;

        while !send_done.load(Ordering::Relaxed) {
            if sender_got_probe.load(Ordering::Relaxed) {
                let since = ack_mode_since.get_or_insert_with(Instant::now);
                if since.elapsed() >= ACK_TIMEOUT {
                    debug!("ACK timeout, assuming peer finished");
                    break;
                }
                if let Err(e) = send_ack(&send_transport, peer_addr) {
                    debug!(%e, "ACK send failed");
                }
            } else if let Err(e) = send_probe(&send_transport, peer_addr) {
                debug!(%e, "Probe send failed");
            }

            thread::sleep(Duration::from_millis(PUNCH_INTERVAL_MS));
        }
    });

    // Receiver: main thread
    let mut buf = [0u8; 1024];
    loop {
        let (len, src_addr) = transport.recv_from(&mut buf)?;
        let data = &buf[..len];

        if src_addr != peer_addr {
            continue;
        }

        if is_probe(data) {
            debug!("Received probe, switching to ACK mode");
            got_probe.store(true, Ordering::Relaxed);
            // Also send an ACK immediately, don't wait for sender loop
            send_ack(transport, peer_addr)?;
        } else if is_ack(data) {
            // Peer got our probe/ACK and confirmed.
            // Send ACK back so peer can also finish.
            send_ack(transport, peer_addr)?;
            info!("Hole punched!");
            done.store(true, Ordering::Relaxed);
            break;
        }
    }

    sender.join().map_err(|_| "Sender thread panicked")?;

    Ok(())
}
