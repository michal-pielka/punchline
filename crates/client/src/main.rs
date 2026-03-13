use punchline_client::signal::pair_with_peer;
use punchline_client::stun::get_external_address;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stun_address: std::net::SocketAddr = std::env::var("STUN_ADDRESS")?.parse()?;
    let signal_address: std::net::SocketAddr = std::env::var("SIGNAL_ADDRESS")?.parse()?;

    let public_key = std::env::var("MY_PUB_KEY")?;
    let peer_public_key = std::env::var("PEER_PUB_KEY")?;

    let external_addr = get_external_address(stun_address)?;
    println!("My address: {:?}", external_addr);

    let peer = pair_with_peer(external_addr, public_key, peer_public_key, signal_address)?;
    println!(
        "Peer addr: {}, peer public_key: {}",
        peer.target_external_addr, peer.target_public_key
    );

    Ok(())
}
