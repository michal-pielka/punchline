use std::net::SocketAddr;

pub struct PairRequest {
    // Used to let the server know the "hole's" address
    pub external_addr: SocketAddr,
    // String for now, prolly needs a refactorings
    // Used to identify both parties of the communication - TODO: signatures
    pub public_key: String,
    pub target_public_key: String,
}

pub struct PairResponse {
    pub target_external_addr: SocketAddr,
    pub target_public_key: String,
}
