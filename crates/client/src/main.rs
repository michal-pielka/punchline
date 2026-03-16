use std::path::PathBuf;
use std::sync::mpsc::{self, Sender};
use std::thread::{self, JoinHandle};

use anyhow::Context;
use clap::{CommandFactory, Parser};
use punchline_client::cli::{Args, Command};
use punchline_client::config::Config;
use punchline_client::tui::{App, AppEvent, PeerInfo};
use punchline_client::{config, handshake, identity, message, peers, punch, signal, stun};
use tracing::info;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let log_level = if args.quiet {
        None
    } else {
        match args.verbose {
            0 => Some(tracing::Level::INFO),
            1 => Some(tracing::Level::DEBUG),
            _ => Some(tracing::Level::TRACE),
        }
    };

    if let Some(level) = log_level {
        tracing_subscriber::fmt().with_max_level(level).init();
    }

    match args.command {
        Command::Keygen { force } => identity::generate(args.identity_path, force),
        Command::Pubkey => identity::print_pubkey(args.identity_path),
        Command::Config { action } => config::handle(action),
        Command::Peers { action } => peers::handle(action),
        Command::Status => status(args.identity_path),
        Command::Completions { shell } => {
            clap_complete::generate(
                shell,
                &mut Args::command(),
                "punchline",
                &mut std::io::stdout(),
            );
            Ok(())
        }
        Command::Connect {
            peer_key,
            stun,
            signal,
        } => connect(args.identity_path, &peer_key, stun, signal),
    }
}

fn status(identity_path: Option<PathBuf>) -> anyhow::Result<()> {
    let cfg = config::load_config().unwrap_or(Config {
        stun_server: None,
        signal_server: None,
    });

    match identity::load_identity(identity_path) {
        Ok((_secret, public)) => {
            println!("Identity:       {}", hex::encode(public));
        }
        Err(_) => println!("Identity:       not found"),
    }

    match config::default_config_path() {
        Ok(path) if path.exists() => println!("Config:         {}", path.display()),
        Ok(_) => println!("Config:         not found"),
        Err(_) => println!("Config:         not found"),
    }

    match cfg.stun_server {
        Some(addr) => {
            let reachable = stun::get_external_addr(addr).is_ok();
            let tag = if reachable {
                "reachable"
            } else {
                "unreachable"
            };
            println!("STUN server:    {addr} [{tag}]");
        }
        None => println!("STUN server:    not configured"),
    }

    match cfg.signal_server {
        Some(addr) => {
            let reachable =
                std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(3))
                    .is_ok();
            let tag = if reachable {
                "reachable"
            } else {
                "unreachable"
            };
            println!("Signal server:  {addr} [{tag}]");
        }
        None => println!("Signal server:  not configured"),
    }

    let count = peers::load().map(|p| p.peers.len()).unwrap_or(0);
    println!("Known peers:    {count}");

    Ok(())
}

fn spawn_terminal_thread(tx: Sender<AppEvent>) -> JoinHandle<()> {
    thread::spawn(move || {
        loop {
            if let Ok(event) = crossterm::event::read() {
                let app_event = match event {
                    crossterm::event::Event::Key(key) => AppEvent::Key(key),
                    crossterm::event::Event::Resize(w, h) => AppEvent::Resize(w, h),
                    _ => continue,
                };
                if tx.send(app_event).is_err() {
                    break;
                }
            }
        }
    })
}

fn connect(
    identity_path: Option<PathBuf>,
    peer_key: &str,
    stun_addr: Option<String>,
    signal_addr: Option<String>,
) -> anyhow::Result<()> {
    let cfg = config::load_config().unwrap_or(Config {
        stun_server: None,
        signal_server: None,
    });

    let stun_addr = match stun_addr {
        Some(s) => s.parse().context("Invalid --stun address")?,
        None => cfg.stun_server.ok_or_else(|| {
            anyhow::anyhow!("No STUN address. Use --stun or set 'stun_server' in config.toml")
        })?,
    };

    let signal_addr = match signal_addr {
        Some(s) => s.parse().context("Invalid --signal address")?,
        None => cfg.signal_server.ok_or_else(|| {
            anyhow::anyhow!("No signal address. Use --signal or set 'signal_server' in config.toml")
        })?,
    };

    let (secret_key, public_key) = identity::load_identity(identity_path)
        .context("No identity found. Run 'punchline keygen' first.")?;
    info!(public_key = %hex::encode(public_key), "Identity loaded");

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
    info!(%external_addr, "Discovered external address");

    let peer = signal::pair_with_peer(external_addr, &public_key, &peer_public_key, signal_addr)
        .context("Signaling failed")?;
    let peer_addr = peer.target_external_addr;
    info!(%peer_addr, peer_key = %peer.target_public_key, "Paired with peer");

    punch::establish(&sock, peer_addr).context("Hole punching failed")?;
    info!("Connection established, ready for messages");

    let noise =
        handshake::exchange_keys(&secret_key, &public_key, &peer_public_key, &sock, peer_addr)
            .context("Key exchange failed")?;

    // App channel
    // tx: recv thread -> main thread
    // tx_term: term thread -> main thread
    // rx: main thread <- recv thread | term thread
    let (tx, rx) = mpsc::channel::<AppEvent>();
    let tx_term = tx.clone();

    // Message channel
    // tx_out: main thread -> send thread
    // rx_out: send thread <- main thread
    let (tx_out, rx_out) = mpsc::channel::<String>();

    // Spawn message recv, send threads
    message::start(noise, &sock, tx, rx_out, peer_addr)?;

    // Spawn terminal thread
    let _terminal_handle = spawn_terminal_thread(tx_term);

    // TUI - main thread
    let terminal = ratatui::init();
    let app = App::new(PeerInfo {
        alias: peer_alias,
        public_key: peer_key_resolved,
        addr: peer_addr.to_string(),
    });
    let result = app.run(terminal, rx, tx_out);
    ratatui::restore();

    result
}
