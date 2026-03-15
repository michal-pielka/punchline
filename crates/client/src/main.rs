use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use ed25519_dalek::VerifyingKey;
use punchline_client::cli::{Args, Command};
use punchline_client::config::Config;
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
        Command::Connect {
            peer_key,
            stun,
            signal,
        } => connect(args.identity_path, &peer_key, stun, signal),
        Command::Peers { action } => peers::handle(action),
    }
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

    let identity = identity::load_identity(identity_path)
        .context("No identity found. Run 'punchline keygen' first.")?;
    let public_key = identity.verifying_key();
    info!(public_key = %hex::encode(public_key.to_bytes()), "Identity loaded");

    let peer_public_key_bytes: [u8; 32] = hex::decode(peer_key)
        .context("Peer key is not valid hex")?
        .try_into()
        .map_err(|_| anyhow::anyhow!("Peer key must be 32 bytes (64 hex chars)"))?;
    let peer_public_key = VerifyingKey::from_bytes(&peer_public_key_bytes)
        .context("Peer key is not a valid Ed25519 public key")?;

    let (external_addr, sock) =
        stun::get_external_addr(stun_addr).context("STUN discovery failed")?;
    info!(%external_addr, "Discovered external address");

    let peer = signal::pair_with_peer(
        &identity,
        external_addr,
        &public_key,
        &peer_public_key,
        signal_addr,
    )
    .context("Signaling failed")?;
    let peer_addr = peer.target_external_addr;
    info!(%peer_addr, peer_key = %peer.target_public_key, "Paired with peer");

    punch::establish(&sock, peer_addr).context("Hole punching failed")?;
    info!("Connection established, ready for messages");

    let shared_secret = handshake::exchange_keys(&identity, &peer_public_key, &sock, peer_addr)
        .context("Key exchange failed")?;

    let is_initiator = public_key.as_bytes() < peer_public_key.as_bytes();
    message::start(&shared_secret, &sock, peer_addr, is_initiator)?;

    Ok(())
}
