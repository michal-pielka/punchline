use clap::Parser;

#[derive(Parser)]
#[command(version, about = "Punchline signaling server")]
pub struct Args {
    /// Bind address
    #[arg(short, long, default_value = "0.0.0.0")]
    pub address: String,

    /// Bind port
    #[arg(short, long, default_value_t = 8743)]
    pub port: u16,

    /// Increase log verbosity (-v debug, -vv trace)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Suppress all output
    #[arg(short, long)]
    pub quiet: bool,
}
