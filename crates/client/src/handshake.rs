use std::net::SocketAddr;

use punchline_proto::transport::Transport;
use snow::{HandshakeState, TransportState};

const NOISE_PATTERN: &str = "Noise_IK_25519_ChaChaPoly_SHA256";
const PUNCH_PROBE: u8 = 0x00;
const PUNCH_ACK: u8 = 0x01;

fn recv_handshake<T: Transport>(
    transport: &T,
    peer_addr: SocketAddr,
    buf: &mut [u8],
) -> anyhow::Result<usize> {
    loop {
        let (len, addr) = transport.recv_from(buf)?;
        if addr != peer_addr {
            continue;
        }
        // Skip leftover punch packets
        if len > 0 && (buf[0] == PUNCH_PROBE || buf[0] == PUNCH_ACK) {
            continue;
        }

        return Ok(len);
    }
}

// TODO: maybe add some payload instead of passing &[]
pub fn exchange_keys<T: Transport>(
    private_key: &[u8; 32],
    public_key: &[u8; 32],
    peer_public_key: &[u8; 32],
    transport: &T,
    peer_addr: SocketAddr,
) -> anyhow::Result<TransportState> {
    let builder = snow::Builder::new(NOISE_PATTERN.parse()?);
    let mut handshake_state: HandshakeState;
    let is_initiator = u64::from_be_bytes(public_key[..8].try_into()?)
        < u64::from_be_bytes(peer_public_key[..8].try_into()?);

    let mut enc_send_buf = [0u8; 1024];
    let mut enc_recv_buf = [0u8; 1024];
    let mut dec_recv_buf = [0u8; 1024];

    if is_initiator {
        handshake_state = builder
            .local_private_key(private_key)?
            .remote_public_key(peer_public_key)?
            .build_initiator()?;

        // Encrypt
        let len = handshake_state.write_message(&[], &mut enc_send_buf)?;

        // Send
        let _len = transport.send_to(&enc_send_buf[..len], peer_addr)?;

        // Receive
        let len = recv_handshake(transport, peer_addr, &mut enc_recv_buf)?;

        // Decrypt
        handshake_state.read_message(&enc_recv_buf[..len], &mut dec_recv_buf)?;
    } else {
        handshake_state = builder
            .local_private_key(private_key)?
            .remote_public_key(peer_public_key)?
            .build_responder()?;

        // Receive
        let len = recv_handshake(transport, peer_addr, &mut enc_recv_buf)?;

        // Decrypt
        handshake_state.read_message(&enc_recv_buf[..len], &mut dec_recv_buf)?;

        // Encrypt
        let len = handshake_state.write_message(&[], &mut enc_send_buf)?;

        // Send
        let _len = transport.send_to(&enc_send_buf[..len], peer_addr)?;
    };

    Ok(handshake_state.into_transport_mode()?)
}
