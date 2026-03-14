use ed25519_dalek::SigningKey;
use rand_core::OsRng;

pub fn generate_identity() -> SigningKey {
    SigningKey::generate(&mut OsRng)
}
