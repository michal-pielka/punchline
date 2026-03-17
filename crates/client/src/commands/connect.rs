use std::path::PathBuf;
use std::sync::mpsc::{self, Sender};
use std::thread::{self, JoinHandle};

use crate::config::{self, Config};
use crate::tui::{App, AppEvent, PeerInfo};
use crate::{handshake, identity, message, peers, punch, signal, stun, style};
use anyhow::Context;

pub fn handle(
    identity_path: Option<PathBuf>,
    peer_key: &str,
    stun_addr: Option<String>,
    signal_addr: Option<String>,
) -> anyhow::Result<()> {
    let cfg = config::load_config().unwrap_or(Config {
        stun_server: None,
        signal_server: None,
    });

    let stun_addr = resolve_addr(stun_addr, cfg.stun_server, "stun")?;
    let signal_addr = resolve_addr(signal_addr, cfg.signal_server, "signal")?;

    let (tx, rx) = mpsc::channel::<AppEvent>();

    let _terminal_handle = spawn_terminal_thread(tx.clone());
    let _connection_handle = spawn_connection_thread(
        tx,
        identity_path,
        peer_key.to_string(),
        stun_addr,
        signal_addr,
    );

    let terminal = ratatui::init();
    let app = App::new(style::load_style());
    let result = app.run(terminal, rx);
    ratatui::restore();

    result
}

fn resolve_addr(
    cli: Option<String>,
    cfg: Option<std::net::SocketAddr>,
    name: &str,
) -> anyhow::Result<String> {
    match cli {
        Some(s) => Ok(s),
        None => cfg.map(|s| s.to_string()).ok_or_else(|| {
            anyhow::anyhow!("No {name} address. Use --{name} or set '{name}_server' in config.toml")
        }),
    }
}

fn spawn_terminal_thread(tx: Sender<AppEvent>) -> JoinHandle<()> {
    thread::spawn(move || {
        loop {
            if let Ok(crossterm::event::Event::Key(key)) = crossterm::event::read()
                && tx.send(AppEvent::Key(key)).is_err()
            {
                break;
            }
        }
    })
}

fn spawn_connection_thread(
    tx: Sender<AppEvent>,
    identity_path: Option<PathBuf>,
    peer_key: String,
    stun_addr: String,
    signal_addr: String,
) -> JoinHandle<()> {
    thread::spawn(move || {
        if let Err(e) = run_connection(
            tx.clone(),
            identity_path,
            &peer_key,
            &stun_addr,
            &signal_addr,
        ) {
            let _ = tx.send(AppEvent::Error(format!("{e:#}")));
        }
    })
}

fn run_connection(
    tx: Sender<AppEvent>,
    identity_path: Option<PathBuf>,
    peer_key: &str,
    stun_addr: &str,
    signal_addr: &str,
) -> anyhow::Result<()> {
    let stun_addr = stun_addr.parse().context("Invalid STUN address")?;
    let signal_addr = signal_addr.parse().context("Invalid signal address")?;

    let (secret_key, public_key) = identity::load_identity(identity_path)
        .context("No identity found. Run 'punchline keygen' first.")?;

    let peer_key_resolved = peers::resolve_peer_key(peer_key)?;
    let peer_alias = if peer_key_resolved != peer_key {
        Some(peer_key.to_string())
    } else {
        None
    };
    let peer_public_key: [u8; 32] = hex::decode(&peer_key_resolved)
        .context("Peer key is not valid hex")?
        .try_into()
        .map_err(|_| anyhow::anyhow!("Peer key must be 32 bytes (64 hex chars)"))?;

    let (external_addr, sock) =
        stun::get_external_addr(stun_addr).context("STUN discovery failed")?;

    let peer = signal::pair_with_peer(external_addr, &public_key, &peer_public_key, signal_addr)
        .context("Signaling failed")?;
    let peer_addr = peer.target_external_addr;

    punch::establish(&sock, peer_addr).context("Hole punching failed")?;

    let noise =
        handshake::exchange_keys(&secret_key, &public_key, &peer_public_key, &sock, peer_addr)
            .context("Key exchange failed")?;

    let (tx_out, rx_out) = mpsc::channel::<String>();
    message::start(noise, &sock, tx.clone(), rx_out, peer_addr)?;

    let _ = tx.send(AppEvent::Connected {
        peer: PeerInfo {
            alias: peer_alias,
            public_key: peer_key_resolved,
            addr: peer_addr.to_string(),
        },
        tx_out,
    });

    Ok(())
}
