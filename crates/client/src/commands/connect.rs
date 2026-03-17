use std::path::PathBuf;
use std::sync::mpsc::{self, Sender};
use std::thread::{self, JoinHandle};

use crate::config;
use crate::tui::{App, AppEvent, ConnectInfo, PeerInfo};
use crate::{handshake, identity, message, peers, punch, signal, stun, style};
use anyhow::Context;

pub fn handle(
    identity_path: Option<PathBuf>,
    peer_key: &str,
    stun_addr: Option<String>,
    signal_addr: Option<String>,
) -> anyhow::Result<()> {
    let cfg = config::load_config().unwrap_or_default();

    let stun_addr = resolve_addr(stun_addr, cfg.stun_server, "stun")?;
    let signal_addr = resolve_addr(signal_addr, cfg.signal_server, "signal")?;

    let (secret_key, public_key) = identity::load_identity(identity_path.clone())
        .context("No identity found. Run 'punchline keygen' first.")?;

    let peer_key_resolved = peers::resolve_peer_key(peer_key)?;
    let peer_alias = if peer_key_resolved != peer_key {
        Some(peer_key.to_string())
    } else {
        None
    };

    let connect_info = ConnectInfo {
        own_public_key: hex::encode(public_key),
        target_key: peer_key_resolved.clone(),
        target_alias: peer_alias.clone(),
        stun_addr: stun_addr.clone(),
        signal_addr: signal_addr.clone(),
    };

    let (tx, rx) = mpsc::channel::<AppEvent>();

    let _terminal_handle = spawn_terminal_thread(tx.clone());
    let _connection_handle = spawn_connection_thread(
        tx,
        secret_key,
        public_key,
        peer_key_resolved,
        peer_alias,
        stun_addr,
        signal_addr,
    );

    let terminal = ratatui::init();
    let app = App::new(style::load_style(), connect_info);
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
    secret_key: [u8; 32],
    public_key: [u8; 32],
    peer_key_resolved: String,
    peer_alias: Option<String>,
    stun_addr: String,
    signal_addr: String,
) -> JoinHandle<()> {
    thread::spawn(move || {
        if let Err(e) = run_connection(
            tx.clone(),
            secret_key,
            public_key,
            &peer_key_resolved,
            peer_alias.as_deref(),
            &stun_addr,
            &signal_addr,
        ) {
            let _ = tx.send(AppEvent::Error(format!("{e:#}")));
        }
    })
}

fn run_connection(
    tx: Sender<AppEvent>,
    secret_key: [u8; 32],
    public_key: [u8; 32],
    peer_key_resolved: &str,
    peer_alias: Option<&str>,
    stun_addr: &str,
    signal_addr: &str,
) -> anyhow::Result<()> {
    let stun_addr = stun_addr.parse().context("Invalid STUN address")?;
    let signal_addr = signal_addr.parse().context("Invalid signal address")?;

    let peer_public_key: [u8; 32] = hex::decode(peer_key_resolved)
        .context("Peer key is not valid hex")?
        .try_into()
        .map_err(|_| anyhow::anyhow!("Peer key must be 32 bytes (64 hex chars)"))?;

    // STUN Discovery
    let (external_addr, sock) = match stun::get_external_addr(stun_addr) {
        Ok(result) => {
            let _ = tx.send(AppEvent::StepComplete {
                step: 0,
                detail: result.0.to_string(),
            });
            result
        }
        Err(e) => {
            let _ = tx.send(AppEvent::StepFailed {
                step: 0,
                detail: format!("{e:#}"),
            });
            return Err(e).context("STUN discovery failed");
        }
    };

    // Signal Server (connect) + match
    let _ = tx.send(AppEvent::StepComplete {
        step: 1,
        detail: "connected".into(),
    });
    let peer =
        match signal::pair_with_peer(external_addr, &public_key, &peer_public_key, signal_addr) {
            Ok(result) => {
                let _ = tx.send(AppEvent::StepComplete {
                    step: 2,
                    detail: "paired".into(),
                });
                result
            }
            Err(e) => {
                let _ = tx.send(AppEvent::StepFailed {
                    step: 2,
                    detail: format!("{e:#}"),
                });
                return Err(e).context("Signaling failed");
            }
        };
    let peer_addr = peer.target_external_addr;

    // Hole Punch
    match punch::establish(&sock, peer_addr) {
        Ok(()) => {
            let _ = tx.send(AppEvent::StepComplete {
                step: 3,
                detail: "established".into(),
            });
        }
        Err(e) => {
            let _ = tx.send(AppEvent::StepFailed {
                step: 3,
                detail: format!("{e:#}"),
            });
            return Err(e).context("Hole punching failed");
        }
    }

    // Noise Handshake
    let noise = match handshake::exchange_keys(
        &secret_key,
        &public_key,
        &peer_public_key,
        &sock,
        peer_addr,
    ) {
        Ok(result) => {
            let _ = tx.send(AppEvent::StepComplete {
                step: 4,
                detail: "secure".into(),
            });
            result
        }
        Err(e) => {
            let _ = tx.send(AppEvent::StepFailed {
                step: 4,
                detail: format!("{e:#}"),
            });
            return Err(e).context("Key exchange failed");
        }
    };

    let (tx_out, rx_out) = mpsc::channel::<String>();
    message::start(noise, &sock, tx.clone(), rx_out, peer_addr)?;

    let _ = tx.send(AppEvent::Connected {
        peer: PeerInfo {
            alias: peer_alias.map(String::from),
            public_key: peer_key_resolved.to_string(),
            addr: peer_addr.to_string(),
        },
        tx_out,
    });

    Ok(())
}
