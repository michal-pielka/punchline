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

#[derive(Serialize, Deserialize)]
pub struct PairResponse {
    pub target_external_addr: SocketAddr,
    pub target_public_key: String,
}
