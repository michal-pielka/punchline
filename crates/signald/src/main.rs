use std::net::{TcpListener, TcpStream};
use std::thread;
use tungstenite::accept;

const ADDRESS: &str = "0.0.0.0";
const PORT: &str = "8743";

fn handle_connection(stream: TcpStream) {
    let mut ws = accept(stream).expect("WebSocket accept failed");

    loop {
        let msg = ws.read().expect("Read failed");

        if msg.is_text() {
            ws.send(msg).expect("Send failed");
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(format!("{}:{}", ADDRESS, PORT))?;

    for stream in listener.incoming() {
        let stream = stream?;

        thread::spawn(move || handle_connection(stream));
    }

    Ok(())
}
