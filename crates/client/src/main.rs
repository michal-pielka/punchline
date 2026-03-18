use clap::Parser;
use punchline::cli::{Args, Command};
use punchline::commands;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Keygen {
            force,
            identity_path,
        } => commands::keygen::handle(identity_path, force),
        Command::Pubkey { identity_path } => commands::pubkey::handle(identity_path),
        Command::Config { action } => commands::config::handle(action),
        Command::Peers { action } => commands::peers::handle(action),
        Command::Status => commands::status::handle(None),
        Command::Completions { shell } => commands::completions::handle(shell),
        Command::Connect {
            peer_key,
            identity_path,
            stun,
            signal,
        } => commands::connect::handle(identity_path, &peer_key, stun, signal),
    }
}
