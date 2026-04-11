use crate::{CompiletimeError, CompiletimeResult, Expr, Interpreter, Stmt, Token};
use std::{collections::HashMap, ops::Deref};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FunctionType {
    None,
    Function,
}

pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Self {
            interpreter,
            scopes: Vec::new(),
            current_function: FunctionType::None,
        }
    }

    pub fn resolve_statements(&mut self, statements: &[Stmt]) -> CompiletimeResult<()> {
        for stmt in statements {
            self.resolve_statement(stmt)?;
        }

        Ok(())
    }

    fn resolve_statement(&mut self, stmt: &Stmt) -> CompiletimeResult<()> {
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
                name,
                params: _,
                body: _,
            } => {
                self.declare(name)?;
                self.define(name)?;
                self.resolve_function(stmt, FunctionType::Function)?;
                Ok(())
            }
            Print { expression } => self.resolve_expression(expression),
            Return { keyword: _, value } => {
                if self.current_function == FunctionType::None {
                    return Err(CompiletimeError::ReturnOutsideFunction);
                }

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

    pub fn resolve_block(&mut self, block: &[Stmt]) -> CompiletimeResult<()> {
        self.begin_scope();
        let result = self.resolve_statements(block);
        self.end_scope();
        result
    }

    fn resolve_expression(&mut self, expr: &Expr) -> CompiletimeResult<()> {
        use Expr::*;
        match expr {
            Variable { id: _, name } => {
                if self
                    .scopes
                    .last()
                    .is_some_and(|f| f.get(&name.lexeme).is_some_and(|v| *v == false))
                {
                    return Err(CompiletimeError::InitializerError(name.clone()));
                }

                self.resolve_local(expr, name);
            }
            Assign {
                id: _,
                name,
                value,
            } => {
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

    fn resolve_function(
        &mut self,
        function: &Stmt,
        function_type: FunctionType,
    ) -> CompiletimeResult<()> {
        let enclosing_function = self.current_function;
        self.current_function = function_type;

        let result = (|| match function {
            Stmt::Function {
                name: _,
                params,
                body,
            } => {
                self.begin_scope();

                for param in params {
                    self.declare(param)?;
                    self.define(param)?;
                }

                let result = self.resolve_statements(body);
                self.end_scope();
                result
            }
            _ => unreachable!(),
        })();

        self.current_function = enclosing_function;
        result
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn resolve_local(&mut self, expr: &Expr, name: &Token) {
        for (idx, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(&name.lexeme) {
                self.interpreter.resolve(expr, idx);
                return;
            }
        }
    }

    fn declare(&mut self, name: &Token) -> CompiletimeResult<()> {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(&name.lexeme) {
                return Err(CompiletimeError::AlreadyDeclared(name.clone()));
            }
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
