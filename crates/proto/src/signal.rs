use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Serialize, Deserialize)]
pub struct PairRequest {
    // Used to let the server know the "hole's" address
    pub external_addr: SocketAddr,
    // String for now, prolly needs a refactorings
    // Used to identify both parties of the communication - TODO: signatures
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

    pub fn verifying_key(&self) -> Result<VerifyingKey, Box<dyn std::error::Error>> {
        let bytes: [u8; 32] = hex::decode(&self.public_key)?
            .try_into()
            .map_err(|_| "Invalid public key")?;
        let verifying_key = VerifyingKey::from_bytes(&bytes)?;

        Ok(verifying_key)
    }

    pub fn target_verifying_key(&self) -> Result<VerifyingKey, Box<dyn std::error::Error>> {
        let bytes: [u8; 32] = hex::decode(&self.target_public_key)?
            .try_into()
            .map_err(|_| "Invalid public key")?;
        let target_verifying_key = VerifyingKey::from_bytes(&bytes)?;

        Ok(target_verifying_key)
    }

    pub fn signature(&self) -> Result<Signature, Box<dyn std::error::Error>> {
        let bytes: [u8; 64] = hex::decode(&self.signature)?
            .try_into()
            .map_err(|_| "Invalid signature")?;

        let signature = Signature::from_bytes(&bytes);
        Ok(signature)
    }
}

#[derive(Serialize, Deserialize)]
pub struct PairResponse {
    pub target_external_addr: SocketAddr,
    pub target_public_key: String,
}
