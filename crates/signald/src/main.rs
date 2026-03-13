use std::net::TcpListener;
use tungstenite::accept;

const ADDRESS: &str = "0.0.0.0";
const PORT: &str = "8743";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(format!("{}:{}", ADDRESS, PORT))?;

    if let Some(stream) = listener.incoming().next() {
        let stream = stream?;
        let mut ws = accept(stream)?;

        loop {
            let msg = ws.read()?;

            if msg.is_text() {
                ws.send(msg)?;
            }
        }
    }

    Ok(())
}
