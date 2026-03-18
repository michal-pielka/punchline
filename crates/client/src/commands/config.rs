use crate::cli::ConfigAction;
use crate::config::{default_config_path, load_config};

pub fn handle(action: ConfigAction) -> anyhow::Result<()> {
    match action {
        ConfigAction::Path => {
            println!("{}", default_config_path()?.display());
        }
        ConfigAction::Show => {
            let path = default_config_path()?;
            match load_config() {
                Ok(cfg) => {
                    println!(
                        "stun_server = {}",
                        cfg.stun_server
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| "(not set)".into())
                    );
                    println!(
                        "signal_server = {}",
                        cfg.signal_server
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| "(not set)".into())
                    );
                }
                Err(_) => {
                    eprintln!("No config file found at {}", path.display());
                }
            }
        }
    }

    Ok(())
}
