use crate::{CompiletimeError, CompiletimeResult, Expr, Stmt, Token, TokenType};
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
    pub fn parse(&mut self) -> CompiletimeResult<Vec<Stmt>> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            if let Some(stmt) = self.declaration() {
                statements.push(stmt);
            }
        }

        Ok(statements)
    }

    fn declaration(&mut self) -> Option<Stmt> {
        let res = if self.match_token(&[Var]) {
            self.var_declaration()
        } else {
            self.statement()
        };

        match res {
            Ok(stmt) => Some(stmt),
            Err(e) => {
                eprintln!("{}", e);
                self.synchronize();
                None
            }
        }
    }

    fn var_declaration(&mut self) -> CompiletimeResult<Stmt> {
        let name = self.consume(&Identifier, "Expect variable name.")?;

        let initializer = if self.match_token(&[Equal]) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(&Semicolon, "Expect ';' after variable declaration")?;

        Ok(Stmt::Var {
            name,
            initializer: Box::new(initializer),
        })
    }

    fn statement(&mut self) -> CompiletimeResult<Stmt> {
        if self.match_token(&[Print]) {
            self.print_statement()
        } else if self.match_token(&[LeftBrace]) {
            Ok(Stmt::Block {
                statements: self.block()?,
            })
        } else {
            self.expression_statement()
        }
    }

    fn print_statement(&mut self) -> CompiletimeResult<Stmt> {
        let value = self.expression()?;
        self.consume(&Semicolon, "Expect ';' after value")?;

        Ok(Stmt::Print {
            expression: value.into_box(),
        })
    }

    fn block(&mut self) -> CompiletimeResult<Vec<Stmt>> {
        let mut statements = Vec::new();

        while !self.check(&RightBrace) && !self.is_at_end() {
            if let Some(decl) = self.declaration() {
                statements.push(decl);
            }
        }

        self.consume(&RightBrace, "Expect '}' after block")?;

        Ok(statements)
    }

    fn expression_statement(&mut self) -> CompiletimeResult<Stmt> {
        let expr = self.expression()?;
        self.consume(&Semicolon, "Expect ';' after expression")?;
        Ok(Stmt::Expression {
            expression: expr.into_box(),
        })
    }

    fn expression(&mut self) -> CompiletimeResult<Expr> {
        self.assignment()
    }

    fn equality(&mut self) -> CompiletimeResult<Expr> {
        let mut expr = self.comparison()?;
        while self.match_token(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: expr.into_box(),
                operator,
                right: right.into_box(),
            }
        }

        Ok(expr)
    }

    fn assignment(&mut self) -> CompiletimeResult<Expr> {
        let expr = self.equality()?;

        if self.match_token(&[Equal]) {
            let equals = self.previous();
            let value = self.assignment()?;

            if let Expr::Variable { name } = expr {
                Ok(Expr::Assign {
                    name,
                    value: value.into_box(),
                })
            } else {
                Err(CompiletimeError::ParseError {
                    line: equals.line,
                    message: "Invalid assignment target".to_string(),
                    lexeme: equals.lexeme,
                })
            }
        } else {
            Ok(expr)
        }
    }

    fn comparison(&mut self) -> CompiletimeResult<Expr> {
        let mut expr = self.term()?;

        while self.match_token(&[Greater, GreaterEqual, Less, LessEqual]) {
            let operator = self.previous();
            let right = self.term()?.into_box();
            expr = Expr::Binary {
                left: expr.into_box(),
                operator,
                right,
            }
        }

        Ok(expr)
    }

    fn term(&mut self) -> CompiletimeResult<Expr> {
        let mut expr = self.factor()?;

        while self.match_token(&[Minus, Plus]) {
            let operator = self.previous();
            let right = self.factor()?.into_box();
            expr = Expr::Binary {
                left: expr.into_box(),
                operator,
                right,
            }
        }

        Ok(expr)
    }

    fn factor(&mut self) -> CompiletimeResult<Expr> {
        let mut expr = self.unary()?;

        while self.match_token(&[Slash, Star]) {
            let operator = self.previous();
            let right = self.unary()?.into_box();
            expr = Expr::Binary {
                left: expr.into_box(),
                operator,
                right,
            }
        }

        Ok(expr)
    }

    fn unary(&mut self) -> CompiletimeResult<Expr> {
        if self.match_token(&[Bang, Minus]) {
            let operator = self.previous();
            let right = self.unary()?.into_box();
            Ok(Expr::Unary { operator, right })
        } else {
            Ok(self.primary()?)
        }
    }

    fn primary(&mut self) -> CompiletimeResult<Expr> {
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
            let expression = self.expression()?.into_box();
            self.consume(&RightParen, "Expect ')' after expression")?;
            Ok(Expr::Grouping { expression })
        } else if self.match_token(&[Identifier]) {
            Ok(Expr::Variable {
                name: self.previous(),
            })
        } else {
            Err(self.error("Unknown token"))
        }
    }

    fn consume(&mut self, token_type: &TokenType, msg: &str) -> CompiletimeResult<Token> {
        if self.check(token_type) {
            Ok(self.advance().clone())
        } else {
            Err(self.error(msg))
        }
    }

    fn error(&self, msg: &str) -> CompiletimeError {
        let token = self.peek();
        CompiletimeError::ParseError {
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

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }

            match self.peek().token_type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => {
                    self.advance();
                }
            }
        }
    }
}
