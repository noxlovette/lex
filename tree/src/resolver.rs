use crate::{CompiletimeError, CompiletimeResult, Expr, Interpreter, Stmt, Token};
use std::{collections::HashMap, ops::Deref};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FunctionType {
    None,
    Function,
    Initializer,
    Method,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ClassType {
    None,
    Class,
    Subclass,
}

pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
    current_class: ClassType,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Self {
            interpreter,
            scopes: Vec::new(),
            current_function: FunctionType::None,
            current_class: ClassType::None,
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
            Print { expression } => self.resolve_expression(expression),
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
                if let Some(else_branch) = else_branch {
                    self.resolve_statement(else_branch)?;
                }
                Ok(())
            }
            While { condition, body } => {
                self.resolve_expression(condition)?;
                self.resolve_statement(body)
            }
            Function { name, .. } => {
                self.declare(name)?;
                self.define(name)?;
                self.resolve_function(stmt, FunctionType::Function)
            }
            Return { value, .. } => {
                if self.current_function == FunctionType::None {
                    return Err(CompiletimeError::ReturnOutsideFunction);
                }

                if let Some(value) = value {
                    if self.current_function == FunctionType::Initializer {
                        return Err(CompiletimeError::ReturnFromInitializer);
                    }
                    self.resolve_expression(value)?;
                }

                Ok(())
            }
            Class {
                name,
                super_class,
                methods,
            } => {
                let enclosing_class = self.current_class;
                self.current_class = ClassType::Class;

                self.declare(name)?;
                self.define(name)?;

                if let Some(super_class) = super_class {
                    if super_class
                        .as_superclass()
                        .is_some_and(|token| token.lexeme == name.lexeme)
                    {
                        return Err(CompiletimeError::InheritSelf(name.clone()));
                    }

                    self.current_class = ClassType::Subclass;
                    self.resolve_expression(super_class)?;

                    self.begin_scope();
                    self.scopes
                        .last_mut()
                        .expect("scope was just created")
                        .insert("super".to_string(), true);
                }

                self.begin_scope();
                self.scopes
                    .last_mut()
                    .expect("scope was just created")
                    .insert("this".to_string(), true);

                for method in methods {
                    let declaration = match method {
                        Stmt::Function { name, .. } if name.lexeme == "init" => {
                            FunctionType::Initializer
                        }
                        Stmt::Function { .. } => FunctionType::Method,
                        _ => unreachable!(),
                    };
                    self.resolve_function(method, declaration)?;
                }

                self.end_scope();
                if super_class.is_some() {
                    self.end_scope();
                }

                self.current_class = enclosing_class;
                Ok(())
            }
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
            Variable { name, .. } => {
                if self
                    .scopes
                    .last()
                    .is_some_and(|scope| scope.get(&name.lexeme).is_some_and(|value| !*value))
                {
                    return Err(CompiletimeError::InitializerError(name.clone()));
                }

                self.resolve_local(expr, name);
            }
            Assign { name, value, .. } => {
                self.resolve_expression(value)?;
                self.resolve_local(expr, name);
            }
            Binary { left, right, .. } | Logical { left, right, .. } => {
                self.resolve_expression(left)?;
                self.resolve_expression(right)?;
            }
            Call {
                callee, arguments, ..
            } => {
                self.resolve_expression(callee)?;
                for arg in arguments {
                    self.resolve_expression(arg)?;
                }
            }
            Grouping { expression } => self.resolve_expression(expression)?,
            Unary { right, .. } => self.resolve_expression(right)?,
            Literal { .. } => {}
            Get { object, .. } => self.resolve_expression(object)?,
            Set { object, value, .. } => {
                self.resolve_expression(value)?;
                self.resolve_expression(object)?;
            }
            This { keyword, .. } => {
                if self.current_class == ClassType::None {
                    return Err(CompiletimeError::ThisOutsideClass);
                }
                self.resolve_local(expr, keyword);
            }
            Super { keyword, .. } => {
                match self.current_class {
                    ClassType::None => return Err(CompiletimeError::SuperOutsideClass),
                    ClassType::Class => return Err(CompiletimeError::SuperWithoutSuperclass),
                    ClassType::Subclass => {}
                }
                self.resolve_local(expr, keyword);
            }
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

        let result = match function {
            Stmt::Function { params, body, .. } => {
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
        };

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
        for (depth, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(&name.lexeme) {
                self.interpreter.resolve(expr, depth);
                return;
            }
        }
    }

    fn declare(&mut self, name: &Token) -> CompiletimeResult<()> {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(&name.lexeme) {
                return Err(CompiletimeError::AlreadyDeclared(name.clone()));
            }
            scope.insert(name.lexeme.clone(), false);
        }

        Ok(())
    }

    fn define(&mut self, name: &Token) -> CompiletimeResult<()> {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.lexeme.clone(), true);
        }

        Ok(())
    }
}
