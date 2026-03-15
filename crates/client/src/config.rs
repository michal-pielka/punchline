use std::{net::SocketAddr, path::PathBuf};

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub stun_address: Option<SocketAddr>,
    pub signal_address: Option<SocketAddr>,
}

pub fn default_config_path() -> anyhow::Result<PathBuf> {
    Ok(dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
        .join("punchline")
        .join("config.toml"))
}

pub fn load_config() -> anyhow::Result<Config> {
    let config_path = default_config_path()?;
    let content = std::fs::read_to_string(config_path)?;

    Ok(toml::from_str(&content)?)
}
