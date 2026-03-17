use crate::cli::Args;
use clap::CommandFactory;
use clap_complete::Shell;

pub fn handle(shell: Shell) -> anyhow::Result<()> {
    clap_complete::generate(
        shell,
        &mut Args::command(),
        "punchline",
        &mut std::io::stdout(),
    );
    Ok(())
}
