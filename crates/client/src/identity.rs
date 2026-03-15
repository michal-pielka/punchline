use ed25519_dalek::SigningKey;
use punchline_proto::crypto;
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

    let key = crypto::generate_identity();
    write_identity(&key, Some(key_path.clone()))?;

    let public_key = key.verifying_key();
    eprintln!("Keypair generated at {}", key_path.display());
    eprintln!("Public key: {}", hex::encode(public_key.to_bytes()));

    Ok(())
}

pub fn print_pubkey(path: Option<PathBuf>) -> anyhow::Result<()> {
    let identity = load_identity(path)?;
    let public_key = identity.verifying_key();

    println!("{}", hex::encode(public_key.to_bytes()));

    Ok(())
}
