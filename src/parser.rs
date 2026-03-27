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
        } else if self.match_token(&[If]) {
            self.if_statement()
        } else if self.match_token(&[While]) {
            self.while_statement()
        } else if self.match_token(&[For]) {
            self.for_statement()
        } else {
            self.expression_statement()
        }
    }

    fn while_statement(&mut self) -> CompiletimeResult<Stmt> {
        self.consume(&LeftParen, "Expect '(' after 'while'")?;
        let condition = self.expression()?;
        self.consume(&RightParen, "Expect ')' after 'while'")?;
        let body = self.statement()?;

        Ok(Stmt::While {
            condition: condition.into_box(),
            body: body.into_box(),
        })
    }

    fn print_statement(&mut self) -> CompiletimeResult<Stmt> {
        let value = self.expression()?;
        self.consume(&Semicolon, "Expect ';' after value")?;

        Ok(Stmt::Print {
            expression: value.into_box(),
        })
    }

    fn for_statement(&mut self) -> CompiletimeResult<Stmt> {
        self.consume(&LeftParen, "Expect '(' after 'for'")?;

        let initializer = if self.match_token(&[Semicolon]) {
            None
        } else if self.match_token(&[Var]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition = if !self.check(&Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(&Semicolon, "Expect ';' after loop condition")?;

        let increment = if !self.check(&RightParen) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(&RightParen, "Expect ')' after for clauses")?;

        let mut body = self.statement()?;

        if let Some(increment) = increment {
            body = Stmt::Block {
                statements: vec![
                    body,
                    Stmt::Expression {
                        expression: increment.into_box(),
                    },
                ],
            }
        }

        let condition = condition.unwrap_or(Expr::Literal { value: true.into() });
        body = Stmt::While {
            condition: condition.into_box(),
            body: body.into_box(),
        };

        if let Some(initializer) = initializer {
            body = Stmt::Block {
                statements: vec![initializer, body],
            }
        }

        Ok(body)
    }

    fn if_statement(&mut self) -> CompiletimeResult<Stmt> {
        self.consume(&LeftParen, "Expect '(' after 'if'")?;
        let condition = self.expression()?;
        self.consume(&RightParen, "Expect ')' after if condition")?;

        let then_branch = self.statement()?;
        let else_branch = if self.match_token(&[Else]) {
            Some(self.statement()?.into_box())
        } else {
            None
        };

        Ok(Stmt::If {
            condition: condition.into_box(),
            then_branch: then_branch.into_box(),
            else_branch,
        })
    }

    fn or_expr(&mut self) -> CompiletimeResult<Expr> {
        let mut expr = self.and_expr()?;
        while self.match_token(&[Or]) {
            let operator = self.previous();
            let right = self.and_expr()?;
            expr = Expr::Logical {
                left: expr.into_box(),
                operator,
                right: right.into_box(),
            }
        }
        Ok(expr)
    }

    fn and_expr(&mut self) -> CompiletimeResult<Expr> {
        let mut expr = self.equality()?;

        while self.match_token(&[And]) {
            let operator = self.previous();
            let right = self.equality()?;
            expr = Expr::Logical {
                left: expr.into_box(),
                operator,
                right: right.into_box(),
            }
        }

        Ok(expr)
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
        let expr = self.or_expr()?;

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
