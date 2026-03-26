use crate::{Expr, InterpreterError, InterpreterResult, Token, TokenType};
use crate::{Literal, TokenType::*};

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
    fn expression(&mut self) -> InterpreterResult<Expr> {
        Ok(self.equality()?)
    }

    fn equality(&mut self) -> InterpreterResult<Expr> {
        let mut expr = self.comparison()?;
        while self.match_token(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: expr.rc(),
                operator: operator,
                right: right.rc(),
            }
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> InterpreterResult<Expr> {
        let mut expr = self.term()?;

        while self.match_token(&[Greater, GreaterEqual, Less, LessEqual]) {
            let operator = self.previous();
            let right = self.term()?.rc();
            expr = Expr::Binary {
                left: expr.rc(),
                operator: operator,
                right,
            }
        }

        Ok(expr)
    }

    fn term(&mut self) -> InterpreterResult<Expr> {
        let mut expr = self.factor()?;

        while self.match_token(&[Minus, Plus]) {
            let operator = self.previous();
            let right = self.factor()?.rc();
            expr = Expr::Binary {
                left: expr.rc(),
                operator: operator,
                right,
            }
        }

        Ok(expr)
    }

    fn factor(&mut self) -> InterpreterResult<Expr> {
        let mut expr = self.unary()?;

        while self.match_token(&[Slash, Star]) {
            let operator = self.previous();
            let right = self.unary()?.rc();
            expr = Expr::Binary {
                left: expr.rc(),
                operator: operator,
                right,
            }
        }

        Ok(expr)
    }

    fn unary(&mut self) -> InterpreterResult<Expr> {
        if self.match_token(&[Bang, Minus]) {
            let operator = self.previous();
            let right = self.unary()?.rc();
            Ok(Expr::Unary { operator, right })
        } else {
            Ok(self.primary()?)
        }
    }

    fn primary(&mut self) -> InterpreterResult<Expr> {
        if self.match_token(&[False]) {
            Ok(Expr::Literal {
                value: Literal::Bool(false),
            })
        } else if self.match_token(&[True]) {
            Ok(Expr::Literal {
                value: Literal::Bool(true),
            })
        } else if self.match_token(&[Nil]) {
            Ok(Expr::Literal {
                value: Literal::Nil,
            })
        } else if self.match_token(&[Number, String]) {
            Ok(Expr::Literal {
                value: self.previous().literal.unwrap_or_default(),
            })
        } else if self.match_token(&[LeftParen]) {
            let expression = self.expression()?.rc();
            self.consume(&RightParen, "Expect ')' after expression")?;
            Ok(Expr::Grouping { expression })
        } else {
            Err(self.error("Unknown token"))
        }
    }

    fn consume(&mut self, token_type: &TokenType, msg: &str) -> InterpreterResult<Token> {
        if self.check(&token_type) {
            Ok(self.advance().clone())
        } else {
            Err(self.error(msg))
        }
    }

    fn error(&self, msg: &str) -> InterpreterError {
        let token = self.peek();
        InterpreterError::ParseError {
            line: token.line,
            message: msg.to_string(),
            lexeme: token.lexeme.clone(),
        }
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

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn previous(&self) -> Token {
        self.tokens.get(self.current - 1).unwrap().clone()
    }
}
