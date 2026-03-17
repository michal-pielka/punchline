use crate::cli::ConfigAction;
use crate::config;

pub fn handle(action: ConfigAction) -> anyhow::Result<()> {
    config::handle(action)
}
