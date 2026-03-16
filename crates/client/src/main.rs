use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

use clap::{CommandFactory, Parser};
use punchline_client::cli::{Args, Command};
use punchline_client::config::Config;
use punchline_client::tui::AppEvent;
use punchline_client::{config, identity, peers, stun};

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let log_level = if args.quiet {
        None
    } else {
        match args.verbose {
            0 => Some(tracing::Level::INFO),
            1 => Some(tracing::Level::DEBUG),
            _ => Some(tracing::Level::TRACE),
        }
    };

    if let Some(level) = log_level {
        tracing_subscriber::fmt().with_max_level(level).init();
    }

    match args.command {
        Command::Keygen { force } => identity::generate(args.identity_path, force),
        Command::Pubkey => identity::print_pubkey(args.identity_path),
        Command::Config { action } => config::handle(action),
        Command::Peers { action } => peers::handle(action),
        Command::Status => status(args.identity_path),
        Command::Completions { shell } => {
            clap_complete::generate(
                shell,
                &mut Args::command(),
                "punchline",
                &mut std::io::stdout(),
            );
            Ok(())
        }
        Command::Connect {
            peer_key,
            stun,
            signal,
        } => connect(args.identity_path, &peer_key, stun, signal),
    }
}

fn status(identity_path: Option<PathBuf>) -> anyhow::Result<()> {
    let cfg = config::load_config().unwrap_or(Config {
        stun_server: None,
        signal_server: None,
    });

    match identity::load_identity(identity_path) {
        Ok((_secret, public)) => {
            println!("Identity:       {}", hex::encode(public));
        }
        Err(_) => println!("Identity:       not found"),
    }

    match config::default_config_path() {
        Ok(path) if path.exists() => println!("Config:         {}", path.display()),
        Ok(_) => println!("Config:         not found"),
        Err(_) => println!("Config:         not found"),
    }

    match cfg.stun_server {
        Some(addr) => {
            let reachable = stun::get_external_addr(addr).is_ok();
            let tag = if reachable {
                "reachable"
            } else {
                "unreachable"
            };
            println!("STUN server:    {addr} [{tag}]");
        }
        None => println!("STUN server:    not configured"),
    }

    match cfg.signal_server {
        Some(addr) => {
            let reachable =
                std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(3))
                    .is_ok();
            let tag = if reachable {
                "reachable"
            } else {
                "unreachable"
            };
            println!("Signal server:  {addr} [{tag}]");
        }
        None => println!("Signal server:  not configured"),
    }

    let count = peers::load().map(|p| p.peers.len()).unwrap_or(0);
    println!("Known peers:    {count}");

    Ok(())
}

fn connect(
    identity_path: Option<PathBuf>,
    peer_key: &str,
    stun_addr: Option<String>,
    signal_addr: Option<String>,
) -> anyhow::Result<()> {
    let (tx, rx) = mpsc::channel::<AppEvent>();

    // Terminal event thread
    let tx_term = tx.clone();
    thread::spawn(move || todo!("read crossterm events, send into tx_term"));

    // TUI
    let terminal = ratatui::init();
    // let app = App::new();
    // let result = app.run(terminal, rx);
    ratatui::restore();

    // result
    Ok(())
}
