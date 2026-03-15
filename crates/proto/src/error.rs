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

    #[error("buffer too short for STUN header")]
    StunBufferTooShort,

    #[error("invalid STUN magic cookie")]
    StunInvalidCookie,

    #[error("not a STUN binding response")]
    StunNotBindingResponse,

    #[error("IPv6 not supported")]
    StunIpv6Unsupported,

    #[error("missing XOR-MAPPED-ADDRESS attribute")]
    StunMissingAddress,

    #[error("invalid XOR-MAPPED-ADDRESS attribute")]
    StunInvalidAddress,
}
