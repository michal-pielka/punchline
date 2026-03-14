use std::io;
use std::net::SocketAddr;
use std::thread;

use chacha20poly1305::{AeadCore, ChaCha20Poly1305, KeyInit, Nonce, aead::Aead};
use punchline_proto::transport::Transport;
use punchline_proto::udp::UdpTransport;
use rand_core::OsRng;
use tracing::{debug, error, info};
use x25519_dalek::SharedSecret;

const MSG_PREFIX: u8 = 0x02;

fn send_loop(
    cipher: ChaCha20Poly1305,
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

        let mut message_plain = vec![MSG_PREFIX];
        message_plain.extend_from_slice(trimmed.as_bytes());

        let nonce = ChaCha20Poly1305::generate_nonce(OsRng);
        let message_encrypted = cipher.encrypt(&nonce, message_plain.as_ref()).unwrap(); // TODO: error

        let mut packet = Vec::new();
        packet.extend_from_slice(&nonce);
        packet.extend_from_slice(&message_encrypted);

        transport.send_to(&packet, peer_addr)?;
        debug!("Sent: {trimmed}");
    }
}

fn recv_loop(
    cipher: ChaCha20Poly1305,
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

        let nonce = Nonce::from_slice(&buf[..12]);
        let message_encrypted = &buf[12..];
        let message_plain = cipher.decrypt(nonce, message_encrypted).unwrap(); // TODO: error
        let message = String::from_utf8_lossy(&message_plain);
        info!("{message}");
    }
}

pub fn start(
    shared_secret: &SharedSecret,
    transport: &UdpTransport,
    peer_addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    let send_transport = transport.try_clone()?;

    let key_bytes = shared_secret.as_bytes();
    let key = chacha20poly1305::Key::from_slice(key_bytes);
    let send_cipher = ChaCha20Poly1305::new(key);
    let recv_cipher = send_cipher.clone();

    thread::spawn(move || {
        if let Err(e) = send_loop(send_cipher, send_transport, peer_addr) {
            error!(%e, "Send loop error");
        }
    });

    recv_loop(recv_cipher, transport, peer_addr)?;

    Ok(())
}
