use crate::identity;
use std::path::PathBuf;

pub fn handle(path: Option<PathBuf>, force: bool) -> anyhow::Result<()> {
    identity::generate(path, force)
}
