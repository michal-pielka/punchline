use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use punchline_client::config::Config;
use punchline_client::{config, handshake, identity, message, peers, punch, signal, stun};
use tracing::info;

fn main() -> anyhow::Result<()> {
    // let app_state = tui::AppState {};

    // let app = tui::App {
    //     should_quit: false,
    //     state: app_state,
    // };

    // let terminal = ratatui::init();
    // let app_result = app.run(terminal);
    //
    // ratatui::restore();
    //
    // app_result

    Ok(())
}

// fn main() -> anyhow::Result<()> {
//     let args = Args::parse();
//
//     let log_level = if args.quiet {
//         None
//     } else {
//         match args.verbose {
//             0 => Some(tracing::Level::INFO),
//             1 => Some(tracing::Level::DEBUG),
//             _ => Some(tracing::Level::TRACE),
//         }
//     };
//
//     if let Some(level) = log_level {
//         tracing_subscriber::fmt().with_max_level(level).init();
//     }
//
//     match args.command {
//         Command::Keygen { force } => identity::generate(args.identity_path, force),
//         Command::Pubkey => identity::print_pubkey(args.identity_path),
//         Command::Config { action } => config::handle(action),
//         Command::Peers { action } => peers::handle(action),
//         Command::Status => status(args.identity_path),
//         Command::Completions { shell } => {
//             clap_complete::generate(
//                 shell,
//                 &mut Args::command(),
//                 "punchline",
//                 &mut std::io::stdout(),
//             );
//             Ok(())
//         }
//         Command::Connect {
//             peer_key,
//             stun,
//             signal,
//         } => connect(args.identity_path, &peer_key, stun, signal),
//     }
// }

fn status(identity_path: Option<PathBuf>) -> anyhow::Result<()> {
    let cfg = config::load_config().unwrap_or(Config {
        stun_server: None,
        signal_server: None,
    });

    // Identity
    match identity::load_identity(identity_path) {
        Ok((_secret, public)) => {
            println!("Identity:       {}", hex::encode(public));
        }
        Err(_) => println!("Identity:       not found"),
    }

    // Config
    match config::default_config_path() {
        Ok(path) if path.exists() => println!("Config:         {}", path.display()),
        Ok(_) => println!("Config:         not found"),
        Err(_) => println!("Config:         not found"),
    }

    // STUN server
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

    // Signal server
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

    // Known peers
    let count = peers::load().map(|p| p.peers.len()).unwrap_or(0);
    println!("Known peers:    {count}");

    Ok(())
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

    let peer_key = peers::resolve_peer_key(peer_key)?;
    let peer_public_key: [u8; 32] = hex::decode(&peer_key)
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

    message::start(noise, &sock, peer_addr)?;

    Ok(())
}
