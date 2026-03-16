use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::thread;

use punchline_proto::transport::Transport;
use snow::TransportState;
use tracing::{debug, error, info};

const MSG_PREFIX: u8 = 0x02;

fn send_loop(
    noise: Arc<Mutex<TransportState>>,
    transport: Box<dyn Transport>,
    rx: std::sync::mpsc::Receiver<String>,
    peer_addr: SocketAddr,
) -> anyhow::Result<()> {
    let mut buf = [0u8; 1024];

    loop {
        // Blocking
        let msg = rx.recv()?;
        let msg_bytes = msg.as_bytes();

        // Encrypt msg to buf
        let len = noise.lock().unwrap().write_message(msg_bytes, &mut buf)?;

        // Send encrypted msg (buf)
        let mut packet = vec![MSG_PREFIX];
        packet.extend_from_slice(&buf[..len]);

        transport.send_to(&packet, peer_addr)?;
    }
}

fn recv_loop(
    noise: Arc<Mutex<TransportState>>,
    transport: &dyn Transport,
    peer_addr: SocketAddr,
) -> anyhow::Result<()> {
    let mut buf = [0u8; 1024];
    let mut plaintext = [0u8; 1024];

    loop {
        let (len, src_addr) = transport.recv_from(&mut buf)?;

        if src_addr != peer_addr {
            continue;
        }

        if len < 2 || buf[0] != MSG_PREFIX {
            continue;
        }

        let plain_len = match noise
            .lock()
            .unwrap()
            .read_message(&buf[1..len], &mut plaintext)
        {
            Ok(len) => len,
            Err(_) => {
                debug!("Decryption failed, skipping packet");
                continue;
            }
        };

        let message = String::from_utf8_lossy(&plaintext[..plain_len]);
        info!("{message}");
    }
}

pub fn start(
    noise: TransportState,
    transport: &dyn Transport,
    tx: std::sync::mpsc::Sender<String>,
    rx: std::sync::mpsc::Receiver<String>,
    peer_addr: SocketAddr,
) -> anyhow::Result<()> {
    let send_transport = transport.try_clone()?;
    let noise = Arc::new(Mutex::new(noise));

    let send_noise = noise.clone();
    thread::spawn(move || {
        if let Err(e) = send_loop(send_noise, send_transport, rx, peer_addr) {
            error!(%e, "Send loop error");
        }
    });

    recv_loop(noise, transport, peer_addr)?;

    Ok(())
}
