#[derive(Debug, thiserror::Error)]
pub enum ProtoError {
    #[error("invalid hex encoding: {0}")]
    InvalidHex(#[from] hex::FromHexError),

    #[error("invalid public key: {0}")]
    InvalidKey(#[from] ed25519_dalek::SignatureError),

    #[error("invalid key length")]
    InvalidKeyLength,

    #[error("invalid signature length")]
    InvalidSignatureLength,
}
