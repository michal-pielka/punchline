use punchline_proto::signal::{PairRequest, PairResponse};
use std::net::SocketAddr;
use tracing::{debug, info};
use tungstenite::connect;

pub fn pair_with_peer(
    external_addr: SocketAddr,
    public_key: String,
    peer_public_key: String,
    signal_addr: SocketAddr,
) -> Result<PairResponse, Box<dyn std::error::Error>> {
    debug!(%signal_addr, "Connecting to signal server");
    let (mut sock, _response) = connect(format!("ws://{}", signal_addr))?;

    let pair_request = PairRequest {
        external_addr,
        public_key,
        target_public_key: peer_public_key,
    };

    let json = serde_json::to_string(&pair_request)?;
    sock.send(tungstenite::Message::Text(json.into()))?;

    info!("Pair request sent, waiting for match...");

    let msg = sock.read()?;
    let pair_response = serde_json::from_str::<PairResponse>(msg.to_text()?)?;

    Ok(pair_response)
}
