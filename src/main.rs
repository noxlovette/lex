use clap::Parser;
use lex::Cli;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    cli.exec()?;

    Ok(())
}
