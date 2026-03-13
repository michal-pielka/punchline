use std::net::UdpSocket;

const ADDRESS: &str = "0.0.0.0";
const PORT: &str = "3478";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sock = UdpSocket::bind(format!("{}:{}", ADDRESS, PORT))?;
    let mut buf = [0u8; 1024];

    loop {
        let (len, src_addr) = sock.recv_from(&mut buf).unwrap();

        println!("Received: {len} bytes from {src_addr}");

        let request = &buf[..len];

        if let Err(e) = sock.send_to(request, src_addr) {
            println!("Sending error: {e}");
        }
    }
}
