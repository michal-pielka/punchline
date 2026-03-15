use rand_core::OsRng;
use x25519_dalek::{PublicKey, StaticSecret};

pub fn generate_static_keypair() -> ([u8; 32], [u8; 32]) {
    let secret = StaticSecret::random_from_rng(OsRng);
    let public = PublicKey::from(&secret);
    (secret.to_bytes(), public.to_bytes())
}

pub fn public_key_from_secret(secret: &[u8; 32]) -> [u8; 32] {
    let secret = StaticSecret::from(*secret);
    let public = PublicKey::from(&secret);
    public.to_bytes()
}
