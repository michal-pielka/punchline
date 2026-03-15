use punchline_proto::crypto;
use std::path::PathBuf;

fn default_key_path() -> anyhow::Result<PathBuf> {
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

fn write_identity(secret: &[u8; 32], path: Option<PathBuf>) -> anyhow::Result<()> {
    let key_path = match path {
        Some(p) => p,
        None => default_key_path()?,
    };

    std::fs::write(&key_path, secret)?;
    set_owner_only_permissions(&key_path)?;
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

pub fn generate(path: Option<PathBuf>, force: bool) -> anyhow::Result<()> {
    let key_path = match &path {
        Some(p) => p.clone(),
        None => default_key_path()?,
    };

    if key_path.exists() && !force {
        anyhow::bail!(
            "Key already exists at {}. Use --force to overwrite.",
            key_path.display()
        );
    }

    if let Some(parent) = key_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let (secret, public) = crypto::generate_static_keypair();
    write_identity(&secret, Some(key_path.clone()))?;

    eprintln!("Keypair generated at {}", key_path.display());
    eprintln!("Public key: {}", hex::encode(public));

    Ok(())
}

pub fn print_pubkey(path: Option<PathBuf>) -> anyhow::Result<()> {
    let (_secret, public) = load_identity(path)?;
    println!("{}", hex::encode(public));
    Ok(())
}
