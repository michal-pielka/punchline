use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about = "Punchline P2P encrypted messaging")]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,

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

    /// Connect to a peer
    Connect {
        /// Peer's public key (64 hex chars)
        peer_key: String,

        /// STUN server address
        #[arg(long)]
        stun: String,

        /// Signal server address
        #[arg(long)]
        signal: String,
    },
}
