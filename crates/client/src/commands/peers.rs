use crate::cli::PeersAction;
use crate::peers;

pub fn handle(action: Option<PeersAction>) -> anyhow::Result<()> {
    match action {
        None => {
            let peers = peers::load()?;
            if peers.peers.is_empty() {
                eprintln!("No known peers. Use 'punchline peers add <name> <key>' to add one.");
            } else {
                for (name, key) in &peers.peers {
                    println!("{name} {key}");
                }
            }
        }
        Some(PeersAction::Add { name, key }) => {
            peers::validate_key(&key)?;
            let mut peers = peers::load()?;
            if let Some(existing) = peers.peers.get(&name) {
                anyhow::bail!("Peer '{name}' already exists with key {existing}");
            }
            peers.peers.insert(name.clone(), key);
            peers::save(&peers)?;
            eprintln!("Added peer '{name}'");
        }
        Some(PeersAction::Remove { name }) => {
            let mut peers = peers::load()?;
            if peers.peers.remove(&name).is_none() {
                anyhow::bail!("Peer '{name}' not found");
            }
            peers::save(&peers)?;
            eprintln!("Removed peer '{name}'");
        }
    }

    Ok(())
}
