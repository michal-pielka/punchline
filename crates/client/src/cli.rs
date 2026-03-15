use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about = "Punchline P2P encrypted messaging")]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,

    /// Path to identity key file
    #[arg(short = 'i', long = "identity", global = true)]
    pub identity_path: Option<PathBuf>,

    /// Increase log verbosity (-v debug, -vv trace)
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Suppress all output
    #[arg(short, long, global = true)]
    pub quiet: bool,
}

#[derive(Subcommand)]
pub enum Command {
    /// Generate a new identity keypair
    Keygen {
        /// Overwrite existing keypair without prompting
        #[arg(short, long)]
        force: bool,
    },

    /// Print your public key
    Pubkey,

    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Manage known peers
    Peers {
        #[command(subcommand)]
        action: Option<PeersAction>,
    },

    /// Connect to a peer
    Connect {
        /// Peer's public key (64 hex chars)
        peer_key: String,

        /// STUN server address
        #[arg(short, long)]
        stun: Option<String>,

        /// Signal server address
        #[arg(short = 'g', long)]
        signal: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Print the config file path
    Path,

    /// Show current config values
    Show,
}

#[derive(Subcommand)]
pub enum PeersAction {
    /// Add a peer
    Add {
        /// Nickname for the peer
        name: String,

        /// Peer's public key (64 hex chars)
        key: String,
    },

    /// Remove a peer
    Remove {
        /// Nickname of the peer to remove
        name: String,
    },
}
