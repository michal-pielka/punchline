use clap::Parser;
use punchline_client::cli::{Args, Command};
use punchline_client::commands;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Keygen { force } => commands::keygen::handle(args.identity_path, force),
        Command::Pubkey => commands::pubkey::handle(args.identity_path),
        Command::Config { action } => commands::config::handle(action),
        Command::Peers { action } => commands::peers::handle(action),
        Command::Status => commands::status::handle(args.identity_path),
        Command::Completions { shell } => commands::completions::handle(shell),
        Command::Connect {
            peer_key,
            stun,
            signal,
        } => commands::connect::handle(args.identity_path, &peer_key, stun, signal),
    }
}
