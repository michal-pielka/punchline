use std::net::SocketAddr;
use std::path::PathBuf;

use serde::Deserialize;

#[derive(Deserialize, Default)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_full_config() {
        let toml = r#"
            stun_server = "1.2.3.4:3478"
            signal_server = "5.6.7.8:8743"
        "#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(
            config.stun_server.unwrap(),
            "1.2.3.4:3478".parse::<SocketAddr>().unwrap()
        );
        assert_eq!(
            config.signal_server.unwrap(),
            "5.6.7.8:8743".parse::<SocketAddr>().unwrap()
        );
    }

    #[test]
    fn parse_partial_config() {
        let toml = r#"stun_server = "1.2.3.4:3478""#;
        let config: Config = toml::from_str(toml).unwrap();
        assert!(config.stun_server.is_some());
        assert!(config.signal_server.is_none());
    }

    #[test]
    fn parse_empty_config() {
        let config: Config = toml::from_str("").unwrap();
        assert!(config.stun_server.is_none());
        assert!(config.signal_server.is_none());
    }
}
