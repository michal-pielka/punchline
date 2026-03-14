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
    let mut message = Vec::new();

    // external_addr
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

    // public_key
    message.extend_from_slice(public_key.as_bytes());

    // target_public_key
    message.extend_from_slice(target_public_key.as_bytes());
    signing_key.sign(&message)
}
