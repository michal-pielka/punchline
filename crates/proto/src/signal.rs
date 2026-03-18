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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pair_request_encodes_hex() {
        let addr: SocketAddr = "1.2.3.4:5678".parse().unwrap();
        let req = PairRequest::new(addr, &[0xAB; 32], &[0xCD; 32]);
        assert_eq!(req.public_key, "ab".repeat(32));
        assert_eq!(req.target_public_key, "cd".repeat(32));
    }

    #[test]
    fn public_key_bytes_round_trip() {
        let addr: SocketAddr = "1.2.3.4:5678".parse().unwrap();
        let key = [0x42; 32];
        let req = PairRequest::new(addr, &key, &[0; 32]);
        assert_eq!(req.public_key_bytes().unwrap(), key);
    }

    #[test]
    fn public_key_bytes_invalid_hex() {
        let req = PairRequest {
            external_addr: "1.2.3.4:5678".parse().unwrap(),
            public_key: "ZZZZ".to_string(),
            target_public_key: "aa".repeat(32),
        };
        assert!(matches!(
            req.public_key_bytes(),
            Err(ProtoError::InvalidHex(_))
        ));
    }

    #[test]
    fn public_key_bytes_wrong_length() {
        let req = PairRequest {
            external_addr: "1.2.3.4:5678".parse().unwrap(),
            public_key: "aabb".to_string(), // 2 bytes, not 32
            target_public_key: "aa".repeat(32),
        };
        assert!(matches!(
            req.public_key_bytes(),
            Err(ProtoError::InvalidKeyLength)
        ));
    }

    #[test]
    fn pair_request_json_round_trip() {
        let addr: SocketAddr = "10.0.0.1:9999".parse().unwrap();
        let req = PairRequest::new(addr, &[0x11; 32], &[0x22; 32]);
        let json = serde_json::to_string(&req).unwrap();
        let parsed: PairRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.external_addr, addr);
        assert_eq!(parsed.public_key, req.public_key);
        assert_eq!(parsed.target_public_key, req.target_public_key);
    }

    #[test]
    fn pair_response_json_round_trip() {
        let resp = PairResponse {
            target_external_addr: "10.0.0.2:8888".parse().unwrap(),
            target_public_key: "ff".repeat(32),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let parsed: PairResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.target_external_addr, resp.target_external_addr);
        assert_eq!(parsed.target_public_key, resp.target_public_key);
    }
}
