use std::net::SocketAddr;

use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use rand_core::OsRng;

pub fn generate_identity() -> SigningKey {
    SigningKey::generate(&mut OsRng)
}

pub fn sign_handshake(
    signing_key: &SigningKey,
    external_addr: SocketAddr,
    public_key: &VerifyingKey,
    target_public_key: &VerifyingKey,
) -> Signature {
    let message = build_handshake_message(external_addr, public_key, target_public_key);
    signing_key.sign(&message)
}

pub fn verify_handshake(
    external_addr: SocketAddr,
    public_key: &VerifyingKey,
    target_public_key: &VerifyingKey,
    signature: &Signature,
) -> Result<(), Box<dyn std::error::Error>> {
    let message = build_handshake_message(external_addr, public_key, target_public_key);
    public_key.verify_strict(&message, signature)?;
    Ok(())
}

fn build_handshake_message(
    external_addr: SocketAddr,
    public_key: &VerifyingKey,
    target_public_key: &VerifyingKey,
) -> Vec<u8> {
    let mut message = Vec::new();
    match external_addr {
        SocketAddr::V4(v4) => {
            message.extend_from_slice(&v4.ip().octets());
            message.extend_from_slice(&v4.port().to_be_bytes());
        }
        SocketAddr::V6(v6) => {
            message.extend_from_slice(&v6.ip().octets());
            message.extend_from_slice(&v6.port().to_be_bytes());
        }
    }
    message.extend_from_slice(public_key.as_bytes());
    message.extend_from_slice(target_public_key.as_bytes());
    message
}
