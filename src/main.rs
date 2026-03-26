use clap::Parser;
use lex::{CompiletimeResult, Scanner};
use std::{fs, path::PathBuf};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    /// Path to a .lox file. If ommitted, start the REPL.
    path: Option<PathBuf>,

    /// Print tokens and exit
    #[arg(long)]
    tokens: bool,

    /// Print AST and exit
    #[arg(long)]
    ast: bool,
}

fn main() -> CompiletimeResult<()> {
    let cli = Cli::parse();

    match cli.path {
        Some(ref path) => {
            run_file(path, &cli)?;
        }
        None => {
            run_prompt(&cli)?;
        }
    }

    Ok(())
}

fn run_file(path: &PathBuf, cli: &Cli) -> CompiletimeResult<()> {
    let str = fs::read_to_string(path)?;
    run(&str, cli)?;

    Ok(())
}

fn run(source: &str, cli: &Cli) -> CompiletimeResult<()> {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens();
    todo!()
}

fn run_prompt(cli: &Cli) -> CompiletimeResult<()> {
    use std::io::{self, Write};

    let stdin = io::stdin();
    let mut line = String::new();

    loop {
        print!("> ");
        io::stdout().flush()?;

        line.clear();

        if stdin.read_line(&mut line)? == 0 {
            break; // EOF
        }

        run(&line, cli)?;
    }

    Ok(())
}
