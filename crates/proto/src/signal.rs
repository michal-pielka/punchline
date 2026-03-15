use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

use crate::error::ProtoError;

#[derive(Serialize, Deserialize)]
pub struct PairRequest {
    pub external_addr: SocketAddr,
    pub public_key: String,
    pub target_public_key: String,
}

impl PairRequest {
    pub fn new(
        external_addr: SocketAddr,
        public_key: &[u8; 32],
        target_public_key: &[u8; 32],
    ) -> PairRequest {
        PairRequest {
            external_addr,
            public_key: hex::encode(public_key),
            target_public_key: hex::encode(target_public_key),
        }
    }

    pub fn public_key_bytes(&self) -> Result<[u8; 32], ProtoError> {
        let bytes: [u8; 32] = hex::decode(&self.public_key)?
            .try_into()
            .map_err(|_| ProtoError::InvalidKeyLength)?;
        Ok(bytes)
    }

    pub fn target_public_key_bytes(&self) -> Result<[u8; 32], ProtoError> {
        let bytes: [u8; 32] = hex::decode(&self.target_public_key)?
            .try_into()
            .map_err(|_| ProtoError::InvalidKeyLength)?;
        Ok(bytes)
    }
}

#[derive(Serialize, Deserialize)]
pub struct PairResponse {
    pub target_external_addr: SocketAddr,
    pub target_public_key: String,
}
