use crate::{CompiletimeError, CompiletimeResult, Expr, Interpreter, Stmt, Token};
use std::{collections::HashMap, ops::Deref};

pub struct Resolver {
    interpreter: Interpreter,
    scopes: Vec<HashMap<String, bool>>,
}

impl Resolver {
    pub fn resolve_statement(&mut self, stmt: &Stmt) -> CompiletimeResult<()> {
        use Stmt::*;
        match stmt {
            Block { statements } => self.resolve_block(statements),
            Expression { expression } => self.resolve_expression(expression),
            Var { name, initializer } => {
                self.declare(name)?;
                if let Some(init) = initializer.deref() {
                    self.resolve_expression(init)?;
                }
                self.define(name)
            }
            If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.resolve_expression(condition)?;
                self.resolve_statement(then_branch)?;
                if let Some(eb) = else_branch {
                    self.resolve_statement(eb)?;
                }
                Ok(())
            }
            Function {
                name: _,
                params,
                body,
            } => {
                self.begin_scope();

                for param in params {
                    self.declare(param)?;
                    self.define(param)?;
                }

                for v in body {
                    self.resolve_statement(v)?;
                }

                self.end_scope();

                Ok(())
            }
            Print { expression } => self.resolve_expression(expression),
            Return { keyword: _, value } => {
                if let Some(val) = value {
                    self.resolve_expression(val)?;
                }
                Ok(())
            }
            While { condition, body } => {
                self.resolve_expression(condition)?;
                self.resolve_statement(body)
            }
            _ => unreachable!(),
        }
    }

    fn resolve_block(&mut self, block: &Vec<Stmt>) -> CompiletimeResult<()> {
        self.begin_scope();

        for stmt in block {
            self.resolve_statement(stmt)?;
        }

        self.end_scope();
        Ok(())
    }

    fn resolve_expression(&mut self, expr: &Expr) -> CompiletimeResult<()> {
        use Expr::*;
        match expr {
            Variable { name } => {
                if self
                    .scopes
                    .last()
                    .is_some_and(|f| f.get(&name.lexeme).is_some_and(|v| *v == false))
                {
                    return Err(CompiletimeError::InitializerError(name.clone()));
                }

                self.resolve_local(expr, name);
            }
            Assign { name, value } => {
                self.resolve_expression(value)?;
                self.resolve_local(expr, name);
            }
            Binary {
                left,
                operator: _,
                right,
            }
            | Logical {
                left,
                operator: _,
                right,
            } => {
                self.resolve_expression(left)?;
                self.resolve_expression(right)?;
            }
            Call {
                callee,
                paren: _,
                arguments,
            } => {
                self.resolve_expression(callee)?;

                for arg in arguments {
                    self.resolve_expression(arg)?;
                }
            }
            Grouping { expression } => {
                self.resolve_expression(expression)?;
            }
            Unary { operator: _, right } => {
                self.resolve_expression(right)?;
            }
            Literal { value: _ } => {}

            _ => unreachable!(),
        }
        Ok(())
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn resolve_local(&mut self, expr: &Expr, name: &Token) {
        while let Some((idx, scope)) = self.scopes.iter().rev().enumerate().next() {
            if scope.contains_key(&name.lexeme) {
                self.interpreter.resolve(expr, idx);
            }
        }
    }

    fn declare(&mut self, name: &Token) -> CompiletimeResult<()> {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.lexeme.to_owned(), false);
        }

        Ok(())
    }

    fn define(&mut self, name: &Token) -> CompiletimeResult<()> {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.lexeme.to_owned(), true);
        }

        Ok(())
    }
}
