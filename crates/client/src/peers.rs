use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::cli::PeersAction;

#[derive(Serialize, Deserialize, Default)]
pub struct Peers {
    #[serde(default)]
    pub peers: HashMap<String, String>,
}

fn default_peers_path() -> anyhow::Result<PathBuf> {
    Ok(dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
        .join(".punchline")
        .join("known_peers.toml"))
}

pub fn load() -> anyhow::Result<Peers> {
    let path = default_peers_path()?;
    match std::fs::read_to_string(&path) {
        Ok(content) => Ok(toml::from_str(&content)?),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Peers::default()),
        Err(e) => Err(e.into()),
    }
}

fn save(peers: &Peers) -> anyhow::Result<()> {
    let path = default_peers_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(peers)?;
    std::fs::write(path, content)?;
    Ok(())
}

fn validate_key(key: &str) -> anyhow::Result<()> {
    let _bytes: [u8; 32] = hex::decode(key)
        .map_err(|_| anyhow::anyhow!("Key is not valid hex"))?
        .try_into()
        .map_err(|_| anyhow::anyhow!("Key must be 32 bytes (64 hex chars)"))?;
    Ok(())
}

pub fn resolve_peer_key(peer_key: &str) -> anyhow::Result<String> {
    if let Ok(peers) = load()
        && let Some(hex_key) = peers.peers.get(peer_key)
    {
        return Ok(hex_key.clone());
    }
    Ok(peer_key.to_string())
}

pub fn handle(action: Option<PeersAction>) -> anyhow::Result<()> {
    match action {
        None => {
            let peers = load()?;
            if peers.peers.is_empty() {
                eprintln!("No known peers. Use 'punchline peers add <name> <key>' to add one.");
            } else {
                for (name, key) in &peers.peers {
                    println!("{name} {key}");
                }
            }
        }
        Some(PeersAction::Add { name, key }) => {
            validate_key(&key)?;
            let mut peers = load()?;
            if let Some(existing) = peers.peers.get(&name) {
                anyhow::bail!("Peer '{name}' already exists with key {existing}");
            }
            peers.peers.insert(name.clone(), key);
            save(&peers)?;
            eprintln!("Added peer '{name}'");
        }
        Some(PeersAction::Remove { name }) => {
            let mut peers = load()?;
            if peers.peers.remove(&name).is_none() {
                anyhow::bail!("Peer '{name}' not found");
            }
            save(&peers)?;
            eprintln!("Removed peer '{name}'");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_key_valid() {
        let key = "aa".repeat(32); // 64 hex chars = 32 bytes
        assert!(validate_key(&key).is_ok());
    }

    #[test]
    fn validate_key_invalid_hex() {
        assert!(validate_key("ZZZZ").is_err());
    }

    #[test]
    fn validate_key_wrong_length() {
        let key = "aa".repeat(16);
        assert!(validate_key(&key).is_err());
    }

    #[test]
    fn peers_toml_round_trip() {
        let mut peers = Peers::default();
        peers.peers.insert("alice".to_string(), "ab".repeat(32));
        peers.peers.insert("bob".to_string(), "cd".repeat(32));

        let serialized = toml::to_string(&peers).unwrap();
        let parsed: Peers = toml::from_str(&serialized).unwrap();

        assert_eq!(parsed.peers.get("alice"), Some(&"ab".repeat(32)));
        assert_eq!(parsed.peers.get("bob"), Some(&"cd".repeat(32)));
    }

    #[test]
    fn peers_empty_toml() {
        let parsed: Peers = toml::from_str("").unwrap();
        assert!(parsed.peers.is_empty());
    }
}
