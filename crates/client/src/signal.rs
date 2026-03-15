use std::net::SocketAddr;

use punchline_proto::signal::{PairRequest, PairResponse};
use tracing::{debug, info};

pub fn pair_with_peer(
    external_addr: SocketAddr,
    public_key: &[u8; 32],
    peer_public_key: &[u8; 32],
    signal_addr: SocketAddr,
) -> anyhow::Result<PairResponse> {
    debug!(%signal_addr, "Connecting to signal server");
    let (mut sock, _response) = tungstenite::connect(format!("ws://{}", signal_addr))?;

    let pair_request = PairRequest::new(external_addr, public_key, peer_public_key);

    let json = serde_json::to_string(&pair_request)?;
    sock.send(tungstenite::Message::Text(json.into()))?;

    info!("Pair request sent, waiting for match...");

    let msg = sock.read()?;
    let pair_response = serde_json::from_str::<PairResponse>(msg.to_text()?)?;

    Ok(pair_response)
}
