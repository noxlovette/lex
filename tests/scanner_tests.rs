use lex::Scanner;

fn scan_token_strings(source: &str) -> Vec<String> {
    let scanner = Scanner::new(source);

    scanner
        .scan_tokens()
        .unwrap()
        .iter()
        .map(ToString::to_string)
        .collect()
}

#[test]
fn test_positive_case() {
    let source = include_str!("test_files/scanner/positive.lex");

    let actual_tokens = scan_token_strings(source);
    let expected_tokens = vec![
        "Var var None".to_string(),
        "Identifier var1 None".to_string(),
        "Equal = None".to_string(),
        "Number 6.43 Some(Number(6.43))".to_string(),
        "Semicolon ; None".to_string(),
        "Var var None".to_string(),
        "Identifier counter None".to_string(),
        "Equal = None".to_string(),
        "Number 0 Some(Number(0.0))".to_string(),
        "Semicolon ; None".to_string(),
        "While while None".to_string(),
        "LeftParen ( None".to_string(),
        "Identifier counter None".to_string(),
        "LessEqual <= None".to_string(),
        "Identifier var1 None".to_string(),
        "RightParen ) None".to_string(),
        "LeftBrace { None".to_string(),
        "Print print None".to_string(),
        "LeftParen ( None".to_string(),
        "String \"still smaller\" Some(String(\"still smaller\"))".to_string(),
        "RightParen ) None".to_string(),
        "Semicolon ; None".to_string(),
        "Identifier counter None".to_string(),
        "Equal = None".to_string(),
        "Identifier counter None".to_string(),
        "Plus + None".to_string(),
        "Number 1 Some(Number(1.0))".to_string(),
        "Semicolon ; None".to_string(),
        "RightBrace } None".to_string(),
        "Eof  None".to_string(),
    ];

    assert_eq!(expected_tokens, actual_tokens);
}

#[test]
fn test_unexpected_character() {
    let source = include_str!("test_files/scanner/unexpected_character.lex");
    let scanner = Scanner::new(source);

    let error = match scanner.scan_tokens() {
        Ok(_) => panic!("expected scanner error for unexpected character"),
        Err(error) => error,
    };

    assert_eq!("Unexpected character at line 1", error.to_string());
}

#[test]
fn test_unterminated_string() {
    let source = include_str!("test_files/scanner/unterminated_string.lex");
    let scanner = Scanner::new(source);

    let error = match scanner.scan_tokens() {
        Ok(_) => panic!("expected scanner error for unterminated string"),
        Err(error) => error,
    };

    assert_eq!("Unterminated String 1", error.to_string());
}
