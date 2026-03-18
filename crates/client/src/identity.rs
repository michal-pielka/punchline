use punchline_proto::crypto;
use std::path::PathBuf;

pub fn default_key_path() -> anyhow::Result<PathBuf> {
    Ok(dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
        .join(".punchline")
        .join("id_x25519"))
}

pub fn load_identity(path: Option<PathBuf>) -> anyhow::Result<([u8; 32], [u8; 32])> {
    let key_path = match path {
        Some(p) => p,
        None => default_key_path()?,
    };

    let secret: [u8; 32] = std::fs::read(&key_path)?
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid key format"))?;

    let public = crypto::public_key_from_secret(&secret);
    Ok((secret, public))
}

pub fn write_identity(secret: &[u8; 32], path: &PathBuf) -> anyhow::Result<()> {
    std::fs::write(path, secret)?;
    set_owner_only_permissions(path)?;
    Ok(())
}

#[cfg(unix)]
fn set_owner_only_permissions(path: &PathBuf) -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    Ok(())
}

#[cfg(not(unix))]
fn set_owner_only_permissions(_path: &PathBuf) -> anyhow::Result<()> {
    Ok(())
}
