use std::net::SocketAddr;

pub trait Transport {
    fn send_to(&self, buf: &[u8], addr: SocketAddr) -> Result<usize, std::io::Error>;
    fn recv_from(&self, buf: &mut [u8]) -> Result<(usize, SocketAddr), std::io::Error>;
}
