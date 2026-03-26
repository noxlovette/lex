use crate::{InterpreterError, InterpreterResult, KEYWORDS, Literal, Token, TokenType};

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
            line: 1,
            ..Default::default()
        }
    }

    pub fn scan_tokens(mut self) -> InterpreterResult<Vec<Token>> {
        while !&self.is_at_end() {
            self.start = self.current;
            self.scan_token()?;
        }

        self.tokens
            .push(Token::new(TokenType::Eof, "", None, self.line));

        Ok(self.tokens)
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
        let ch = self.chars[self.current];
        self.current += 1;
        ch
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
        if self.current + 1 >= self.chars.len() {
            '\0'
        } else {
            self.chars[self.current + 1]
        }
    }

    fn string(&mut self) -> InterpreterResult<()> {
        use crate::InterpreterError::StringError;

        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
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
        self.current >= self.chars.len()
    }
}
