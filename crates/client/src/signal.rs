use punchline_proto::signal::{PairRequest, PairResponse};
use std::net::SocketAddr;
use tungstenite::connect;

pub fn pair_with_peer(
    external_addr: SocketAddr,
    public_key: String,
    peer_public_key: String,
    signal_server: SocketAddr,
) -> Result<PairResponse, Box<dyn std::error::Error>> {
    let (mut sock, _response) = connect(format!("ws://{}", signal_server))?;

    let pair_request = PairRequest {
        external_addr,
        public_key,
        target_public_key: peer_public_key,
    };

    let json = serde_json::to_string(&pair_request)?;
    sock.send(tungstenite::Message::Text(json.into()))?;

    let msg = sock.read()?;
    let peer_address = serde_json::from_str::<PairResponse>(msg.to_text()?)?;

    Ok(peer_address)
}
