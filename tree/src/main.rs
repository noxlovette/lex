use clap::Parser;
use tree_walk_interpreter::Cli;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    cli.exec()?;

    Ok(())
}
