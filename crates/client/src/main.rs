use anyhow::Context;
use ed25519_dalek::VerifyingKey;
use punchline_client::handshake;
use punchline_client::identity;
use punchline_client::message;
use punchline_client::punch;
use punchline_client::signal;
use punchline_client::stun;
use punchline_proto::crypto;
use tracing::info;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let stun_addr: std::net::SocketAddr = std::env::var("STUN_ADDRESS")
        .context("STUN_ADDRESS not set")?
        .parse()
        .context("Invalid STUN_ADDRESS")?;
    let signal_addr: std::net::SocketAddr = std::env::var("SIGNAL_ADDRESS")
        .context("SIGNAL_ADDRESS not set")?
        .parse()
        .context("Invalid SIGNAL_ADDRESS")?;

    let identity = match identity::load_identity(None) {
        Ok(key) => key,
        Err(_) => {
            let key = crypto::generate_identity();
            identity::write_identity(&key, None).context("Failed to write identity key")?;
            key
        }
    };
    let public_key = identity.verifying_key();
    info!(public_key = %hex::encode(public_key.to_bytes()), "Identity loaded");

    let peer_public_key_string = std::env::var("PEER_PUB_KEY").context("PEER_PUB_KEY not set")?;
    let peer_public_key_bytes: [u8; 32] = hex::decode(&peer_public_key_string)
        .context("PEER_PUB_KEY is not valid hex")?
        .try_into()
        .map_err(|_| anyhow::anyhow!("PEER_PUB_KEY must be 32 bytes (64 hex chars)"))?;
    let peer_public_key = VerifyingKey::from_bytes(&peer_public_key_bytes)
        .context("PEER_PUB_KEY is not a valid Ed25519 public key")?;

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
