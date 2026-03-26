use crate::{Interpreter, Parser, Scanner};
use clap::Parser as ClapParser;
use std::{fs, path::PathBuf};

#[derive(ClapParser, Debug)]
#[command(version, about)]
pub struct Cli {
    /// Path to a .lox file. If ommitted, start the REPL.
    path: Option<PathBuf>,

    /// Print tokens and exit
    #[arg(long)]
    tokens: bool,

    /// Print AST and exit
    #[arg(long)]
    ast: bool,
}

impl Cli {
    pub fn exec(&self) -> anyhow::Result<()> {
        match self.path {
            Some(ref path) => {
                self.run_file(path)?;
            }
            None => {
                self.run_prompt()?;
            }
        }

        Ok(())
    }

    fn run_file(&self, path: &PathBuf) -> anyhow::Result<()> {
        let str = fs::read_to_string(path)?;
        self.run(&str)?;

        Ok(())
    }

    fn run(&self, source: &str) -> anyhow::Result<()> {
        let scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens()?;
        let mut parser = Parser::new(tokens);
        let statements = parser.parse()?;
        let mut interpreter = Interpreter;

        interpreter.interpret(&statements)?;

        Ok(())
    }

    fn run_prompt(&self) -> anyhow::Result<()> {
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

            self.run(&line)?;
        }

        Ok(())
    }
}
