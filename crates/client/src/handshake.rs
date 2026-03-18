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

#[cfg(test)]
mod tests {
    use super::*;
    use punchline_proto::crypto::generate_static_keypair;

    #[test]
    fn initiator_is_smaller_key() {
        let mut low_key = [0u8; 32];
        low_key[0] = 0x00;
        let mut high_key = [0u8; 32];
        high_key[0] = 0xFF;

        let low_val = u64::from_be_bytes(low_key[..8].try_into().unwrap());
        let high_val = u64::from_be_bytes(high_key[..8].try_into().unwrap());

        assert!(low_val < high_val); // low_key is initiator
    }

    #[test]
    fn noise_handshake_in_memory() {
        let (secret_a, public_a) = generate_static_keypair();
        let (secret_b, public_b) = generate_static_keypair();

        let pattern: snow::params::NoiseParams = NOISE_PATTERN.parse().unwrap();

        // A is initiator
        let mut initiator = snow::Builder::new(pattern.clone())
            .local_private_key(&secret_a)
            .unwrap()
            .remote_public_key(&public_b)
            .unwrap()
            .build_initiator()
            .unwrap();

        let mut responder = snow::Builder::new(pattern)
            .local_private_key(&secret_b)
            .unwrap()
            .remote_public_key(&public_a)
            .unwrap()
            .build_responder()
            .unwrap();

        let mut buf1 = [0u8; 1024];
        let mut buf2 = [0u8; 1024];

        // initiator -> responder
        let len = initiator.write_message(&[], &mut buf1).unwrap();
        responder.read_message(&buf1[..len], &mut buf2).unwrap();

        // responder -> initiator
        let len = responder.write_message(&[], &mut buf1).unwrap();
        initiator.read_message(&buf1[..len], &mut buf2).unwrap();

        // Both should transition to transport mode
        let mut transport_a = initiator.into_transport_mode().unwrap();
        let mut transport_b = responder.into_transport_mode().unwrap();

        // Verify encrypted messaging works
        let msg = b"punchline";
        let len = transport_a.write_message(msg, &mut buf1).unwrap();
        let len = transport_b.read_message(&buf1[..len], &mut buf2).unwrap();
        assert_eq!(&buf2[..len], msg);
    }
}
