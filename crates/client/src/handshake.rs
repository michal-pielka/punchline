use std::net::SocketAddr;

use anyhow::Context;
use ed25519_dalek::{Signature, SigningKey, VerifyingKey};
use punchline_proto::{crypto, transport::Transport};
use tracing::{debug, info};
use x25519_dalek::{PublicKey, SharedSecret};

pub fn exchange_keys<T: Transport>(
    signing_key: &SigningKey,
    verifying_key: &VerifyingKey,
    transport: &T,
    peer_addr: SocketAddr,
) -> anyhow::Result<SharedSecret> {
    let (ephemeral_private, ephemeral_public) = crypto::generate_x25519_keypair();
    debug!("Generated ephemeral X25519 keypair");

    let signature = crypto::sign_ephemeral_key(signing_key, &ephemeral_public);

    let mut packet = Vec::new();
    packet.extend_from_slice(ephemeral_public.as_bytes());
    packet.extend_from_slice(&signature.to_bytes());

    transport
        .send_to(&packet, peer_addr)
        .context("Failed to send ephemeral key")?;
    debug!("Sent signed ephemeral key");

    // Receive peer's ephemeral public and signature
    // Loop to skip leftover punch packets
    let mut buf = [0u8; 1024];
    loop {
        let (len, src_addr) = transport.recv_from(&mut buf)?;
        if src_addr == peer_addr && len == 96 {
            break;
        }
        debug!(len, %src_addr, "Skipping non-handshake packet");
    }
    debug!("Received peer's ephemeral key");

    let peer_ephemeral_public_bytes: [u8; 32] = buf[..32]
        .try_into()
        .context("Invalid ephemeral key bytes")?;
    let peer_ephemeral_public = PublicKey::from(peer_ephemeral_public_bytes);

    let peer_signature_bytes: [u8; 64] =
        buf[32..96].try_into().context("Invalid signature bytes")?;
    let peer_signature = Signature::from_bytes(&peer_signature_bytes);

    crypto::verify_ephemeral_key(verifying_key, &peer_ephemeral_public, &peer_signature)
        .context("Peer ephemeral key signature verification failed")?;
    info!("Peer ephemeral key verified");

    Ok(ephemeral_private.diffie_hellman(&peer_ephemeral_public))
}
