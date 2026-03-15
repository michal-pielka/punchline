#[derive(Debug, thiserror::Error)]
pub enum ProtoError {
    #[error("invalid hex encoding: {0}")]
    InvalidHex(#[from] hex::FromHexError),

    #[error("invalid key length")]
    InvalidKeyLength,

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
