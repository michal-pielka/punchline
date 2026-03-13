use punchline_client::stun::get_external_address;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stun_address: std::net::SocketAddr = std::env::var("STUN_ADDRESS")?.parse()?;
    // let signal_address: std::net::SocketAddr = std::env::var("SIGNAL_ADDRESS")?.parse()?;

    let addr = get_external_address(stun_address);
    println!("My address: {:?}", addr);

    Ok(())
}
