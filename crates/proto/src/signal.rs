use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

use crate::error::ProtoError;

#[derive(Serialize, Deserialize)]
pub struct PairRequest {
    pub external_addr: SocketAddr,
    pub public_key: String,
    pub target_public_key: String,
    pub signature: String,
}

impl PairRequest {
    pub fn new(
        external_addr: SocketAddr,
        public_key: &VerifyingKey,
        target_public_key: &VerifyingKey,
        signature: &Signature,
    ) -> PairRequest {
        PairRequest {
            external_addr,
            public_key: hex::encode(public_key.as_bytes()),
            target_public_key: hex::encode(target_public_key.as_bytes()),
            signature: hex::encode(signature.to_bytes()),
        }
    }

    pub fn verifying_key(&self) -> Result<VerifyingKey, ProtoError> {
        let bytes: [u8; 32] = hex::decode(&self.public_key)?
            .try_into()
            .map_err(|_| ProtoError::InvalidKeyLength)?;
        Ok(VerifyingKey::from_bytes(&bytes)?)
    }

    pub fn target_verifying_key(&self) -> Result<VerifyingKey, ProtoError> {
        let bytes: [u8; 32] = hex::decode(&self.target_public_key)?
            .try_into()
            .map_err(|_| ProtoError::InvalidKeyLength)?;
        Ok(VerifyingKey::from_bytes(&bytes)?)
    }

    pub fn signature(&self) -> Result<Signature, ProtoError> {
        let bytes: [u8; 64] = hex::decode(&self.signature)?
            .try_into()
            .map_err(|_| ProtoError::InvalidSignatureLength)?;
        Ok(Signature::from_bytes(&bytes))
    }
}

#[derive(Serialize, Deserialize)]
pub struct PairResponse {
    pub target_external_addr: SocketAddr,
    pub target_public_key: String,
}
