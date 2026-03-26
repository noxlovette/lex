use crate::Literal;
use std::{collections::HashMap, fmt::Display, sync::LazyLock};
use strum::Display;

#[derive(Clone, Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    literal: Option<Literal>,
    line: usize,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {:?}", self.token_type, self.lexeme, self.literal)?;

        Ok(())
    }
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: &str, literal: Option<Literal>, line: usize) -> Self {
        Self {
            token_type,
            lexeme: lexeme.to_owned(),
            literal,
            line,
        }
    }
}

#[derive(Display, Clone, Copy, Debug, PartialEq)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

pub static KEYWORDS: LazyLock<HashMap<&'static str, TokenType>> = LazyLock::new(|| {
    use super::TokenType::*;
    let mut k = HashMap::new();

    k.insert("and", And);
    k.insert("class", Class);
    k.insert("else", Else);
    k.insert("false", False);
    k.insert("for", For);
    k.insert("fun", Fun);
    k.insert("if", If);
    k.insert("nil", Nil);
    k.insert("or", Or);
    k.insert("print", Print);
    k.insert("return", Return);
    k.insert("super", Super);
    k.insert("this", This);
    k.insert("true", True);
    k.insert("var", Var);
    k.insert("while", While);

    k
});
