use std::path::PathBuf;

use punchline_proto::crypto;

use crate::identity;

pub fn handle(path: Option<PathBuf>, force: bool) -> anyhow::Result<()> {
    let key_path = match &path {
        Some(p) => p.clone(),
        None => identity::default_key_path()?,
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
    identity::write_identity(&secret, &key_path)?;

    eprintln!("Keypair generated at {}", key_path.display());
    eprintln!("Public key: {}", hex::encode(public));

    Ok(())
}
