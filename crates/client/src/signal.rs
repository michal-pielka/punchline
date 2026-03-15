use std::net::SocketAddr;

use ed25519_dalek::{SigningKey, VerifyingKey};
use punchline_proto::{
    crypto::sign_handshake,
    signal::{PairRequest, PairResponse},
};
use tracing::{debug, info};
use tungstenite;

pub fn pair_with_peer(
    signing_key: &SigningKey,
    external_addr: SocketAddr,
    public_key: &VerifyingKey,
    peer_public_key: &VerifyingKey,
    signal_addr: SocketAddr,
) -> anyhow::Result<PairResponse> {
    debug!(%signal_addr, "Connecting to signal server");
    let (mut sock, _response) = tungstenite::connect(format!("ws://{}", signal_addr))?;

    let signature = sign_handshake(signing_key, external_addr, public_key, peer_public_key);

    let pair_request = PairRequest::new(external_addr, public_key, peer_public_key, &signature);

    let json = serde_json::to_string(&pair_request)?;
    sock.send(tungstenite::Message::Text(json.into()))?;

    info!("Pair request sent, waiting for match...");

    let msg = sock.read()?;
    let pair_response = serde_json::from_str::<PairResponse>(msg.to_text()?)?;

    Ok(pair_response)
}
