use std::net::SocketAddr;

use ed25519_dalek::{Signature, SigningKey, VerifyingKey};
use punchline_proto::{crypto, transport::Transport};
use x25519_dalek::{PublicKey, SharedSecret};

pub fn exchange_keys<T: Transport>(
    signing_key: &SigningKey,
    verifying_key: &VerifyingKey,
    transport: &T,
    peer_addr: SocketAddr,
) -> Result<SharedSecret, Box<dyn std::error::Error>> {
    // Create ephemeral keypair
    let (ephemeral_private, ephemeral_public) = crypto::generate_x25519_keypair();

    // Sign ephemeral public using our public ed25519 key
    let signature = crypto::sign_ephemeral_key(signing_key, &ephemeral_public);

    // Construct packet: ephemeral_public + signature
    let mut packet = Vec::new();
    packet.extend_from_slice(ephemeral_public.as_bytes());
    packet.extend_from_slice(&signature.to_bytes());

    transport.send_to(&packet, peer_addr)?;

    // Receive peer's ephemeral public and signature
    // Loop to skip leftover punch packets
    let mut buf = [0u8; 1024];
    let len = loop {
        let (len, src_addr) = transport.recv_from(&mut buf)?;
        if src_addr == peer_addr && len == 96 {
            break len;
        }
    };

    let peer_ephemeral_public_bytes: [u8; 32] = buf[..32].try_into()?;
    let peer_ephemeral_public = PublicKey::from(peer_ephemeral_public_bytes);

    let peer_signature_bytes: [u8; 64] = buf[32..96].try_into()?;
    let peer_signature = Signature::from_bytes(&peer_signature_bytes);

    // Verify peer's signature - match against their ed25519 public key
    crypto::verify_ephemeral_key(verifying_key, &peer_ephemeral_public, &peer_signature)?;

    // Return the negotiated shared secret
    Ok(ephemeral_private.diffie_hellman(&peer_ephemeral_public))
}
