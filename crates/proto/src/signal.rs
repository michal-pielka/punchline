use std::net::SocketAddr;

struct PairRequest {
    // Used to let the server know the "hole's" address
    external_addr: SocketAddr,
    // String for now, prolly needs a refactorings
    // Used to identify both parties of the communication - TODO: signatures
    public_key: String,
    target_public_key: String,
}
