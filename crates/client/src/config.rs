use std::net::SocketAddr;
use std::path::PathBuf;

use serde::Deserialize;

use crate::cli::ConfigAction;

#[derive(Deserialize)]
pub struct Config {
    pub stun_server: Option<SocketAddr>,
    pub signal_server: Option<SocketAddr>,
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

pub fn handle(action: ConfigAction) -> anyhow::Result<()> {
    match action {
        ConfigAction::Path => {
            println!("{}", default_config_path()?.display());
        }
        ConfigAction::Show => {
            let path = default_config_path()?;
            match load_config() {
                Ok(cfg) => {
                    println!(
                        "stun_server = {}",
                        cfg.stun_server
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| "(not set)".into())
                    );
                    println!(
                        "signal_server = {}",
                        cfg.signal_server
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| "(not set)".into())
                    );
                }
                Err(_) => {
                    eprintln!("No config file found at {}", path.display());
                }
            }
        }
    }

    Ok(())
}
