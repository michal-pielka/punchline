use punchline_client::signal::pair_with_peer;
use punchline_client::stun::get_external_address;
use tracing::info;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let stun_address: std::net::SocketAddr = std::env::var("STUN_ADDRESS")?.parse()?;
    let signal_address: std::net::SocketAddr = std::env::var("SIGNAL_ADDRESS")?.parse()?;

    let public_key = std::env::var("MY_PUB_KEY")?;
    let peer_public_key = std::env::var("PEER_PUB_KEY")?;

    let external_addr = get_external_address(stun_address)?;
    info!(%external_addr, "Discovered external address");

    let peer = pair_with_peer(external_addr, public_key, peer_public_key, signal_address)?;
    info!(
        peer_addr = %peer.target_external_addr,
        peer_key = %peer.target_public_key,
        "Paired with peer"
    );

    Ok(())
}
