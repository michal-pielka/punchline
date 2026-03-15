use std::io;
use std::net::SocketAddr;
use std::thread;

use chacha20poly1305::{ChaCha20Poly1305, KeyInit, Nonce, aead::Aead};
use hkdf::Hkdf;
use punchline_proto::transport::Transport;
use punchline_proto::udp::UdpTransport;
use sha2::Sha256;
use tracing::{debug, error, info};
use x25519_dalek::SharedSecret;

const MSG_PREFIX: u8 = 0x02;

fn send_loop(
    cipher: ChaCha20Poly1305,
    transport: UdpTransport,
    peer_addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut counter: u64 = 0;
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

        let mut nonce_bytes = [0u8; 12];
        nonce_bytes[4..].copy_from_slice(&counter.to_be_bytes());
        let nonce = Nonce::from_slice(&nonce_bytes);

        let message_encrypted = cipher.encrypt(nonce, message_plain.as_ref()).unwrap(); // TODO: error

        let mut packet = Vec::new();
        packet.extend_from_slice(nonce);
        packet.extend_from_slice(&message_encrypted);

        transport.send_to(&packet, peer_addr)?;
        debug!("Sent: {trimmed}");

        counter += 1;
    }
}

fn recv_loop(
    cipher: ChaCha20Poly1305,
    transport: &UdpTransport,
    peer_addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut counter: u64 = 0;
    let mut buf = [0u8; 1024];

    loop {
        let (len, src_addr) = transport.recv_from(&mut buf)?;

        if src_addr != peer_addr {
            continue;
        }

        if len < 12 {
            continue;
        }

        let nonce = Nonce::from_slice(&buf[..12]);
        let received_counter = u64::from_be_bytes(buf[4..12].try_into()?);

        if received_counter <= counter {
            debug!("Rejected replay: counter {received_counter} <= {counter}");
            continue;
        }

        let message_encrypted = &buf[12..len];
        let message_plain = cipher.decrypt(nonce, message_encrypted).unwrap(); // TODO: error

        if message_plain.is_empty() || message_plain[0] != MSG_PREFIX {
            continue;
        }

        let message = String::from_utf8_lossy(&message_plain[1..]);
        info!("{message}");

        counter = received_counter;
    }
}

pub fn start(
    shared_secret: &SharedSecret,
    transport: &UdpTransport,
    peer_addr: SocketAddr,
    is_initiator: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let send_transport = transport.try_clone()?;

    let key_bytes = shared_secret.as_bytes();
    let hkdf = Hkdf::<Sha256>::new(None, key_bytes);

    let mut send_key_bytes = [0u8; 32];
    let mut recv_key_bytes = [0u8; 32];

    hkdf.expand(&[1, 3, 3, 7], &mut send_key_bytes).unwrap();
    hkdf.expand(&[6, 9], &mut recv_key_bytes).unwrap();

    if !is_initiator {
        std::mem::swap(&mut send_key_bytes, &mut recv_key_bytes);
    }

    let recv_cipher = ChaCha20Poly1305::new_from_slice(&recv_key_bytes)?;
    let send_cipher = ChaCha20Poly1305::new_from_slice(&send_key_bytes)?;

    thread::spawn(move || {
        if let Err(e) = send_loop(send_cipher, send_transport, peer_addr) {
            error!(%e, "Send loop error");
        }
    });

    recv_loop(recv_cipher, transport, peer_addr)?;

    Ok(())
}
