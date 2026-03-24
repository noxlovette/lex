use std::{collections::HashMap, fmt::Display, sync::LazyLock};

use strum::{Display, EnumString};

use crate::{InterpreterError, InterpreterResult};

#[derive(Default)]
pub struct Scanner<'a> {
    source: &'a str,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    chars: Vec<char>,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.chars().collect(),
            ..Default::default()
        }
    }

    pub fn scan_tokens(&mut self) -> InterpreterResult<&Vec<Token>> {
        while !&self.is_at_end() {
            self.start = self.current;
            self.scan_token()?;
        }

        self.tokens
            .push(Token::new(TokenType::Eof, "", None, self.line));

        Ok(&self.tokens)
    }

    fn scan_token(&mut self) -> InterpreterResult<()> {
        use super::TokenType::*;
        use crate::InterpreterError::TokenError;

        let c = self.advance();
        match c {
            '(' => self.add_token(LeftParen, None),
            ')' => self.add_token(RightParen, None),
            '{' => self.add_token(LeftBrace, None),
            '}' => self.add_token(RightBrace, None),
            ',' => self.add_token(Comma, None),
            '.' => self.add_token(Dot, None),
            '-' => self.add_token(Minus, None),
            '+' => self.add_token(Plus, None),
            ';' => self.add_token(Semicolon, None),
            '*' => self.add_token(Star, None),
            '!' => {
                let token = if self.match_next('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                self.add_token(token, None);
            }
            '=' => {
                let token = if self.match_next('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };
                self.add_token(token, None);
            }
            '<' => {
                let token = if self.match_next('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                self.add_token(token, None);
            }
            '>' => {
                let token = if self.match_next('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                self.add_token(token, None);
            }
            '/' => {
                if self.match_next('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(Slash, None);
                }
            }
            ' ' | '\r' | '\t' => (),
            '\n' => self.line += 1,
            '"' => self.string()?,
            c if c.is_ascii_digit() => self.number()?,
            c if c.is_ascii_alphanumeric() => self.identifier(),
            _ => {
                return Err(TokenError { line: self.line });
            }
        };

        Ok(())
    }

    fn advance(&mut self) -> char {
        self.chars[self.current + 1]
    }

    fn identifier(&mut self) {
        while self.peek().is_ascii_alphanumeric() {
            self.advance();
        }

        let text = &self.source[self.start..self.current];

        let t = match KEYWORDS.get(text) {
            Some(&t) => t,
            None => TokenType::Identifier,
        };

        self.add_token(t, None);
    }

    fn number(&mut self) -> InterpreterResult<()> {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            // consume the dot
            self.advance();

            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let num = self.source[self.start..self.current]
            .parse()
            .map_err(|_| InterpreterError::NumberError { line: self.line })?;

        self.add_token(TokenType::Number, Some(Literal::Number(num)));

        Ok(())
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.chars[self.current + 1]
        }
    }

    fn string(&mut self) -> InterpreterResult<()> {
        use crate::InterpreterError::StringError;

        let peek = self.peek();
        let is_at_end = self.is_at_end();

        while peek != '"' && !is_at_end {
            if peek == '\n' {
                self.line += 1
            }
        }

        if is_at_end {
            return Err(StringError { line: self.line });
        }

        // the closing "
        self.advance();

        let value = self.source[self.start + 1..self.current - 1].to_string();

        self.add_token(TokenType::String, Some(Literal::String(value)));

        Ok(())
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.chars[self.current]
        }
    }

    fn match_next(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }

        if self.chars[self.current] != expected {
            return false;
        }

        self.current += 1;
        true
    }

    fn add_token(&mut self, token_type: TokenType, literal: Option<Literal>) {
        let text = &self.source[self.start..self.current];
        self.tokens
            .push(Token::new(token_type, text, literal, self.line));
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
}

pub struct Token {
    token_type: TokenType,
    lexeme: String,
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

#[derive(Debug, Clone, PartialEq, Display)]
pub enum Literal {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
}

#[derive(Display, Clone, Copy)]
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

static KEYWORDS: LazyLock<HashMap<&'static str, TokenType>> = LazyLock::new(|| {
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
