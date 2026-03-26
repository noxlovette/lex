use std::rc::Rc;

use crate::TokenType::*;
use crate::{Expr, Token, TokenType};

#[derive(Default)]
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            ..Default::default()
        }
    }
}

impl Parser {
    fn expression(&mut self) -> Expr {
        self.equality()
    }

    fn equality(&mut self) -> Expr {
        let mut expr = self.comparison();
        while self.match_token(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison();
            expr = Expr::Binary {
                left: expr.rc(),
                operator: operator,
                right: right.rc(),
            }
        }

        expr
    }

    fn comparison(&mut self) -> Expr {
        let mut expr = self.term();

        while self.match_token(&[Greater, GreaterEqual, Less, LessEqual]) {
            let operator = self.previous().clone();
            let right = self.term().rc();
            expr = Expr::Binary {
                left: expr.rc(),
                operator: operator,
                right,
            }
        }

        expr
    }

    fn term(&mut self) -> Expr {
        let mut expr = self.factor();

        while self.match_token(&[Minus, Plus]) {
            let operator = self.previous().clone();
            let right = self.factor().rc();
            expr = Expr::Binary {
                left: expr.rc(),
                operator: operator,
                right,
            }
        }
        expr
    }

    fn factor(&mut self) -> Expr {
        let mut expr = self.unary();

        while self.match_token(&[Slash, Star]) {
            let operator = self.previous();
            let right = self.unary().rc();
            expr = Expr::Binary {
                left: expr.rc(),
                operator: operator.to_owned(),
                right,
            }
        }
        expr
    }

    fn unary(&self) -> Expr {
        todo!()
    }
}

impl Parser {
    fn match_token(&mut self, types: &[TokenType]) -> bool {
        for t in types {
            if self.check(t) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek().token_type == *token_type
        }
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.current).unwrap()
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            let t = self.tokens.get(self.current).unwrap();
            self.current += 1;
            t
        } else {
            self.previous()
        }
    }

    fn previous(&self) -> &Token {
        self.tokens.get(self.current - 1).unwrap()
    }
}
