use crate::identity;
use std::path::PathBuf;

pub fn handle(path: Option<PathBuf>) -> anyhow::Result<()> {
    identity::print_pubkey(path)
}
