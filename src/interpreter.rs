use std::ops::Deref;

use crate::{Environment, Expr, RuntimeResult, Stmt, TokenType, Value};

pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Environment::new(),
        }
    }
    pub fn interpret(&mut self, statements: &[Stmt]) -> RuntimeResult<()> {
        for stmt in statements {
            self.execute(stmt)?;
        }
        Ok(())
    }

    fn eval(&mut self, expr: &Expr) -> RuntimeResult<Value> {
        use Expr::*;
        use TokenType::*;
        match expr {
            Literal { value } => value.into(),
            Grouping { expression } => self.eval(&expression),
            Unary { operator, right } => {
                let right = self.eval(&right)?;
                match operator.token_type {
                    Minus => Ok((-right)?),
                    Bang => Ok(!right),
                    _ => unimplemented!(),
                }
            }
            Binary {
                left,
                operator,
                right,
            } => {
                let left = self.eval(&left)?;
                let right = self.eval(&right)?;
                match operator.token_type {
                    Minus => Ok((left - right)?),
                    Slash => Ok((left / right)?),
                    Star => Ok((left * right)?),
                    Plus => Ok((left + right)?),
                    Greater => Ok((left > right).into()),
                    GreaterEqual => Ok((left >= right).into()),
                    Less => Ok((left < right).into()),
                    LessEqual => Ok((left <= right).into()),
                    BangEqual => Ok((left != right).into()),
                    EqualEqual => Ok((left == right).into()),
                    _ => unreachable!(),
                }
            }
            Variable { name } => self.environment.get(name),
            _ => unimplemented!(),
        }
    }

    fn execute(&mut self, stmt: &Stmt) -> RuntimeResult<()> {
        match stmt {
            Stmt::Expression { expression } => {
                let _ = self.eval(expression)?;
                Ok(())
            }
            Stmt::Print { expression } => {
                let value = self.eval(expression)?;
                println!("{value}");
                Ok(())
            }
            Stmt::Var { name, initializer } => {
                let value = if let Some(i) = initializer.deref() {
                    Some(self.eval(i)?)
                } else {
                    None
                };

                self.environment.define(name.lexeme.clone(), value);
                Ok(())
            }

            _ => unimplemented!(),
        }
    }
}
