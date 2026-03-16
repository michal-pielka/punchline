use std::net::{SocketAddr, UdpSocket};

use crate::transport::Transport;

pub struct UdpTransport {
    socket: UdpSocket,
}

impl UdpTransport {
    pub fn new(socket: UdpSocket) -> Self {
        Self { socket }
    }
}

impl Transport for UdpTransport {
    fn send_to(&self, buf: &[u8], addr: SocketAddr) -> Result<usize, std::io::Error> {
        self.socket.send_to(buf, addr)
    }

    fn recv_from(&self, buf: &mut [u8]) -> Result<(usize, SocketAddr), std::io::Error> {
        self.socket.recv_from(buf)
    }

    fn local_addr(&self) -> Result<SocketAddr, std::io::Error> {
        self.socket.local_addr()
    }

    fn try_clone(&self) -> Result<Box<dyn Transport>, std::io::Error> {
        Ok(Box::new(Self {
            socket: self.socket.try_clone()?,
        }))
    }

    fn set_read_timeout(&self, dur: Option<std::time::Duration>) -> Result<(), std::io::Error> {
        self.socket.set_read_timeout(dur)
    }
}
