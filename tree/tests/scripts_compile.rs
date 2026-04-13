use std::{fs, path::Path};
use tree_walk_interpreter::{Interpreter, Parser, Resolver, Scanner};

fn compile(source: &str) {
    let scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens().expect("scanner should succeed");
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().expect("parser should succeed");
    let mut interpreter = Interpreter::new();
    let mut resolver = Resolver::new(&mut interpreter);
    resolver
        .resolve_statements(&statements)
        .expect("resolver should succeed");
}

#[test]
fn compiles_all_scripts() {
    let scripts_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../scripts");

    let mut paths: Vec<_> = fs::read_dir(&scripts_dir)
        .expect("scripts directory should exist")
        .map(|entry| entry.expect("script entry should be readable").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "lox"))
        .collect();

    paths.sort();

    assert!(!paths.is_empty(), "expected at least one .lox file in scripts/");

    for path in paths {
        let source = fs::read_to_string(&path).expect("script should be readable");
        compile(&source);
    }
}
