use lex::{AstPrinter, Parser, Scanner};

fn main() -> anyhow::Result<()> {
    let source = "1 + 2 * 4 - 3";
    let scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens()?;
    let mut parser = Parser::new(tokens);
    let expression = parser.parse();

    if let Some(expr) = expression {
        println!("{}", AstPrinter.print(&expr));
    } else {
        eprintln!("Parsing failed")
    }

    Ok(())
}
