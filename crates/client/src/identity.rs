use ed25519_dalek::SigningKey;
use std::path::PathBuf;

fn default_key_path() -> anyhow::Result<PathBuf> {
    Ok(dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
        .join(".punchline")
        .join("id_ed25519"))
}

pub fn load_identity(path: Option<PathBuf>) -> anyhow::Result<SigningKey> {
    let key_path = match path {
        Some(p) => p,
        None => default_key_path()?,
    };

    let bytes: [u8; 32] = std::fs::read(key_path)?
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid key format"))?;

    Ok(SigningKey::from_bytes(&bytes))
}

pub fn write_identity(identity: &SigningKey, path: Option<PathBuf>) -> anyhow::Result<()> {
    let key_path = match path {
        Some(p) => p,
        None => default_key_path()?,
    };

    std::fs::write(key_path, identity.as_bytes())?;
    Ok(())
}
