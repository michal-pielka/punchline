use std::path::PathBuf;

use crate::identity;

pub fn handle(path: Option<PathBuf>) -> anyhow::Result<()> {
    let (_secret, public) = identity::load_identity(path)?;
    println!("{}", hex::encode(public));
    Ok(())
}
