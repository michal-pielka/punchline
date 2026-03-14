use punchline_client::message;
use punchline_client::punch;
use punchline_client::signal;
use punchline_client::stun;
use tracing::info;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let stun_addr: std::net::SocketAddr = std::env::var("STUN_ADDRESS")?.parse()?;
    let signal_addr: std::net::SocketAddr = std::env::var("SIGNAL_ADDRESS")?.parse()?;

    let public_key = std::env::var("MY_PUB_KEY")?;
    let peer_public_key = std::env::var("PEER_PUB_KEY")?;

    let (external_addr, sock) = stun::get_external_addr(stun_addr)?;
    info!(%external_addr, "Discovered external address");

    let peer = signal::pair_with_peer(external_addr, public_key, peer_public_key, signal_addr)?;
    let peer_addr = peer.target_external_addr;
    info!(%peer_addr, peer_key = %peer.target_public_key, "Paired with peer");

    punch::establish(&sock, peer_addr)?;
    info!("Connection established, ready for messages");

    message::start(&sock, peer_addr)
}
