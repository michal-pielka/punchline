use std::path::PathBuf;

use crate::config;
use crate::{identity, peers, stun};

pub fn handle(identity_path: Option<PathBuf>) -> anyhow::Result<()> {
    let cfg = config::load_config().unwrap_or_default();

    match identity::load_identity(identity_path) {
        Ok((_secret, public)) => println!("Identity:       {}", hex::encode(public)),
        Err(_) => println!("Identity:       not found"),
    }

    match config::default_config_path() {
        Ok(path) if path.exists() => println!("Config:         {}", path.display()),
        _ => println!("Config:         not found"),
    }

    match cfg.stun_server {
        Some(addr) => {
            let tag = if stun::test_connection(addr).unwrap_or(false) {
                "reachable"
            } else {
                "unreachable"
            };
            println!("STUN server:    {addr} [{tag}]");
        }
        None => println!("STUN server:    not configured"),
    }

    match cfg.signal_server {
        Some(addr) => {
            let tag =
                if std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(3))
                    .is_ok()
                {
                    "reachable"
                } else {
                    "unreachable"
                };
            println!("Signal server:  {addr} [{tag}]");
        }
        None => println!("Signal server:  not configured"),
    }

    println!(
        "Known peers:    {}",
        peers::load().map(|p| p.peers.len()).unwrap_or(0)
    );

    Ok(())
}
