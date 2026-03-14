use ed25519_dalek::SigningKey;
use std::path::PathBuf;

pub fn load_identity(path: Option<PathBuf>) -> Result<SigningKey, Box<dyn std::error::Error>> {
    let key_path = path.unwrap_or_else(|| {
        dirs::home_dir()
            .expect("Could not find home directory")
            .join(".punchline")
            .join("id_ed25519")
    });

    let bytes: [u8; 32] = std::fs::read(key_path)?
        .try_into()
        .map_err(|_| "Invalid key format")?;

    Ok(SigningKey::from_bytes(&bytes))
}

pub fn write_identity(
    identity: &SigningKey,
    path: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let key_path = path.unwrap_or_else(|| {
        dirs::home_dir()
            .expect("Could not find home directory")
            .join(".punchline")
            .join("id_ed25519")
    });

    std::fs::write(key_path, identity.as_bytes())?;
    Ok(())
}
