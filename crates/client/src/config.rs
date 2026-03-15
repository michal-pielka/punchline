use std::path::PathBuf;

pub fn default_config_path() -> anyhow::Result<PathBuf> {
    Ok(dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
        .join("punchline")
        .join("config.toml"))
}

pub fn load_config() {}
