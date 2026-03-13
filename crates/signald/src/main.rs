use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use tungstenite::accept;

use punchline_proto::signal::{PairRequest, PairResponse};

const ADDRESS: &str = "0.0.0.0";
const PORT: &str = "8743";

type WsStream = tungstenite::WebSocket<TcpStream>;

fn handle_connection(
    stream: TcpStream,
    pending_peers: Arc<Mutex<HashMap<String, (PairRequest, WsStream)>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut ws = accept(stream)?;
    let msg = ws.read()?;

    let pair_request = msg
        .to_text()
        .ok()
        .and_then(|p| serde_json::from_str::<PairRequest>(p).ok())
        .ok_or("Invalid pair request.")?;

    let mut map = pending_peers.lock().unwrap();

    // Is target already connected?
    if let Some((target_pair_request, target_ws)) = map.remove(&pair_request.target_public_key) {
        // Send PairResponse ("Go!" message) to both peers
        // send_pair_response(target_pair_request, target_ws);
        // send_pair_response(pair_request, ws);
    } else {
        map.insert(pair_request.public_key.clone(), (pair_request, ws));
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(format!("{}:{}", ADDRESS, PORT))?;

    // HashMap for O(1) lookup
    let pending_peers: Arc<Mutex<HashMap<String, (PairRequest, WsStream)>>> =
        Arc::new(Mutex::new(HashMap::new()));

    for stream in listener.incoming() {
        let stream = stream?;

        let p = pending_peers.clone();
        thread::spawn(move || handle_connection(stream, p));
    }

    Ok(())
}
