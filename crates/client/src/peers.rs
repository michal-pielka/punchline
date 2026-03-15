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
        .join("peers.toml"))
}

pub fn load_peers() -> anyhow::Result<Peers> {
    let config_path = default_peers_path()?;
    let content = std::fs::read_to_string(config_path)?;

    Ok(toml::from_str(&content)?)
}

pub fn handle(action: PeersAction) {}
