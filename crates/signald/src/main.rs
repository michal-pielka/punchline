use punchline_proto::crypto::verify_handshake;
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use tracing::{debug, error, info};
use tungstenite::accept;

use punchline_proto::signal::{PairRequest, PairResponse};

const ADDRESS: &str = "0.0.0.0";
const PORT: &str = "8743";

type WsStream = tungstenite::WebSocket<TcpStream>;

fn send_pair_response(
    response: &PairResponse,
    ws: &mut WsStream,
) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string(response)?;
    ws.send(tungstenite::Message::Text(json.into()))?;
    Ok(())
}

fn handle_connection(
    stream: TcpStream,
    pending_peers: Arc<Mutex<HashMap<String, (PairRequest, WsStream)>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("New connection");
    let mut ws = accept(stream)?;
    let msg = ws.read()?;

    let pair_request = msg
        .to_text()
        .ok()
        .and_then(|p| serde_json::from_str::<PairRequest>(p).ok())
        .ok_or("Invalid pair request.")?;

    // Verify the signature
    debug!(from = %pair_request.public_key, "Verifying signature");
    let verifying_key = pair_request.verifying_key()?;
    let target_verifying_key = pair_request.target_verifying_key()?;
    let signature = pair_request.signature()?;
    verify_handshake(
        pair_request.external_addr,
        &verifying_key,
        &target_verifying_key,
        &signature,
    )?;

    info!(
        from = %pair_request.public_key,
        to = %pair_request.target_public_key,
        "Signature verified, pair request accepted"
    );

    // Check for mutual match: target is waiting AND wants to talk to us
    let is_mutual = {
        let map = pending_peers.lock().unwrap();
        map.get(&pair_request.target_public_key)
            .is_some_and(|(waiting, _)| waiting.target_public_key == pair_request.public_key)
    };

    if is_mutual {
        let (target_pair_request, mut target_ws) = {
            let mut map = pending_peers.lock().unwrap();
            map.remove(&pair_request.target_public_key).unwrap()
        };

        let pair_response = PairResponse {
            target_external_addr: target_pair_request.external_addr,
            target_public_key: target_pair_request.public_key,
        };

        let target_pair_response = PairResponse {
            target_external_addr: pair_request.external_addr,
            target_public_key: pair_request.public_key,
        };

        info!(
            peer_a = %pair_response.target_public_key,
            peer_b = %target_pair_response.target_public_key,
            "Match found"
        );

        if let Err(e) = send_pair_response(&pair_response, &mut ws) {
            error!(%e, "Failed to send response to initiator");
        }
        if let Err(e) = send_pair_response(&target_pair_response, &mut target_ws) {
            error!(%e, "Failed to send response to waiting peer");
        }
    } else {
        let mut map = pending_peers.lock().unwrap();
        info!(
            peer = %pair_request.public_key,
            target = %pair_request.target_public_key,
            "Waiting for match"
        );
        map.insert(pair_request.public_key.clone(), (pair_request, ws));
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let listener = TcpListener::bind(format!("{}:{}", ADDRESS, PORT))?;

    let pending_peers: Arc<Mutex<HashMap<String, (PairRequest, WsStream)>>> =
        Arc::new(Mutex::new(HashMap::new()));

    info!("Signal server listening on {ADDRESS}:{PORT}");

    for stream in listener.incoming() {
        let stream = stream?;

        let p = pending_peers.clone();
        thread::spawn(move || {
            if let Err(e) = handle_connection(stream, p) {
                error!(%e, "Connection error");
            }
        });
    }

    Ok(())
}
