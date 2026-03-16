use std::net::SocketAddr;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use punchline_proto::transport::Transport;
use snow::TransportState;

use crate::tui::AppEvent;

const MSG_PREFIX: u8 = 0x02;
const KEEPALIVE_PREFIX: u8 = 0x03;
const KEEPALIVE_INTERVAL: Duration = Duration::from_secs(10);
const DISCONNECT_TIMEOUT: Duration = Duration::from_secs(30);

fn send_encrypted(
    noise: &Arc<Mutex<TransportState>>,
    transport: &dyn Transport,
    peer_addr: SocketAddr,
    prefix: u8,
    payload: &[u8],
) -> bool {
    let mut buf = [0u8; 1024];
    let len = match noise.lock().unwrap().write_message(payload, &mut buf) {
        Ok(len) => len,
        Err(_) => return false,
    };

    let mut packet = vec![prefix];
    packet.extend_from_slice(&buf[..len]);
    transport.send_to(&packet, peer_addr).is_ok()
}

fn send_loop(
    noise: Arc<Mutex<TransportState>>,
    transport: Box<dyn Transport>,
    rx: Receiver<String>,
    peer_addr: SocketAddr,
) {
    loop {
        match rx.recv_timeout(KEEPALIVE_INTERVAL) {
            Ok(msg) => {
                send_encrypted(&noise, &*transport, peer_addr, MSG_PREFIX, msg.as_bytes());
            }
            Err(RecvTimeoutError::Timeout) => {
                send_encrypted(&noise, &*transport, peer_addr, KEEPALIVE_PREFIX, &[]);
            }
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
}

fn recv_loop(
    noise: Arc<Mutex<TransportState>>,
    transport: Box<dyn Transport>,
    tx: Sender<AppEvent>,
    peer_addr: SocketAddr,
) {
    use std::time::Instant;

    let mut buf = [0u8; 1024];
    let mut plaintext = [0u8; 1024];
    let mut last_received = Instant::now();

    loop {
        let (len, src_addr) = match transport.recv_from(&mut buf) {
            Ok(r) => r,
            Err(e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                if last_received.elapsed() >= DISCONNECT_TIMEOUT {
                    let _ = tx.send(AppEvent::PeerDisconnected);
                    break;
                }
                continue;
            }
            Err(_) => break,
        };

        if src_addr != peer_addr || len < 2 {
            continue;
        }

        let prefix = buf[0];
        if prefix != MSG_PREFIX && prefix != KEEPALIVE_PREFIX {
            continue;
        }

        let plain_len = match noise
            .lock()
            .unwrap()
            .read_message(&buf[1..len], &mut plaintext)
        {
            Ok(len) => len,
            Err(_) => continue,
        };

        last_received = Instant::now();

        // Keepalives are decrypted to advance nonce but not displayed
        if prefix == KEEPALIVE_PREFIX {
            continue;
        }

        let msg = String::from_utf8_lossy(&plaintext[..plain_len]).to_string();
        if tx.send(AppEvent::MessageReceived(msg)).is_err() {
            break;
        }
    }
}

pub fn start(
    noise: TransportState,
    transport: &dyn Transport,
    tx: Sender<AppEvent>,
    rx_outgoing: Receiver<String>,
    peer_addr: SocketAddr,
) -> anyhow::Result<()> {
    let send_transport = transport.try_clone()?;
    let recv_transport = transport.try_clone()?;
    recv_transport.set_read_timeout(Some(Duration::from_secs(5)))?;
    let noise = Arc::new(Mutex::new(noise));

    let send_noise = noise.clone();
    thread::spawn(move || {
        send_loop(send_noise, send_transport, rx_outgoing, peer_addr);
    });

    thread::spawn(move || {
        recv_loop(noise, recv_transport, tx, peer_addr);
    });

    Ok(())
}
