use std::net::UdpSocket;

use crate::transport::Transport;

pub struct UdpTransport {
    socket: UdpSocket,
}

impl Transport for UdpTransport {
    fn send_to(&self, buf: &[u8], addr: std::net::SocketAddr) -> Result<usize, std::io::Error> {
        self.socket.send_to(buf, addr)
    }

    fn recv_from(&self, buf: &mut [u8]) -> Result<(usize, std::net::SocketAddr), std::io::Error> {
        self.socket.recv_from(buf)
    }

    fn local_addr(&self) -> Result<std::net::SocketAddr, std::io::Error> {
        self.socket.local_addr()
    }
}
