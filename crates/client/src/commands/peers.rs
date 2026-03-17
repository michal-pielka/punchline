use crate::cli::PeersAction;
use crate::peers;

pub fn handle(action: Option<PeersAction>) -> anyhow::Result<()> {
    peers::handle(action)
}
