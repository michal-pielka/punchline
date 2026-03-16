use std::net::SocketAddr;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use punchline_proto::transport::Transport;
use snow::TransportState;

use crate::tui::AppEvent;

const MSG_PREFIX: u8 = 0x02;

fn send_loop(
    noise: Arc<Mutex<TransportState>>,
    transport: Box<dyn Transport>,
    rx: Receiver<String>,
    peer_addr: SocketAddr,
) {
    let mut buf = [0u8; 1024];

    while let Ok(msg) = rx.recv() {
        let len = match noise
            .lock()
            .unwrap()
            .write_message(msg.as_bytes(), &mut buf)
        {
            Ok(len) => len,
            Err(_) => continue,
        };

        let mut packet = vec![MSG_PREFIX];
        packet.extend_from_slice(&buf[..len]);

        let _ = transport.send_to(&packet, peer_addr);
    }
}

fn recv_loop(
    noise: Arc<Mutex<TransportState>>,
    transport: Box<dyn Transport>,
    tx: Sender<AppEvent>,
    peer_addr: SocketAddr,
) {
    let mut buf = [0u8; 1024];
    let mut plaintext = [0u8; 1024];

    loop {
        let (len, src_addr) = match transport.recv_from(&mut buf) {
            Ok(r) => r,
            Err(_) => break,
        };

        if src_addr != peer_addr || len < 2 || buf[0] != MSG_PREFIX {
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
