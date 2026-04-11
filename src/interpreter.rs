use crate::{
    Environment, EvalResult, Expr, Function, IsTruthy, RuntimeControl, RuntimeError, RuntimeResult,
    Stmt, Token, TokenType, Value,
};
use std::{cell::RefCell, collections::HashMap, ops::Deref, rc::Rc};

#[derive(Debug)]
pub struct Interpreter {
    pub(crate) environment: Rc<RefCell<Environment>>,
    globals: Rc<RefCell<Environment>>,
    locals: HashMap<usize, usize>,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Rc::new(RefCell::new(Environment::new()));
        Self {
            environment: globals.clone(),
            globals,
            locals: HashMap::new(),
        }
    }

    pub fn interpret(&mut self, statements: &[Stmt]) -> RuntimeResult<()> {
        for stmt in statements {
            self.execute(stmt)?;
        }
        Ok(())
    }

    pub fn resolve(&mut self, expr: &Expr, depth: usize) {
        let id = expr
            .id()
            .expect("resolver only stores locals for variable and assignment expressions");
        self.locals.insert(id, depth);
    }

    fn eval(&mut self, expr: &Expr) -> EvalResult<Value> {
        use Expr::*;
        use TokenType::*;
        match expr {
            Literal { value } => Ok(value.into()),
            Grouping { expression } => self.eval(expression),
            Unary { operator, right } => {
                let right = self.eval(right)?;
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
                let left = self.eval(left)?;
                let right = self.eval(right)?;
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
            Variable { name, .. } => Ok(self.look_up_var(name, expr)?),
            Assign { name, value, .. } => {
                let value = self.eval(value)?;

                let expr_id = expr.id().expect("assignment expressions always have an id");
                if let Some(distance) = self.locals.get(&expr_id) {
                    self.environment
                        .borrow_mut()
                        .assign_at(*distance, name, value.clone())?;
                } else {
                    self.globals.borrow_mut().assign(name, &value)?;
                }

                Ok(value)
            }
            Logical {
                left,
                operator,
                right,
            } => {
                let left = self.eval(left)?;
                if operator.token_type == TokenType::Or {
                    if left.is_truthy() {
                        Ok(left)
                    } else {
                        self.eval(right)
                    }
                } else {
                    if !left.is_truthy() {
                        Ok(left)
                    } else {
                        self.eval(right)
                    }
                }
            }
            Call {
                callee,
                paren,
                arguments,
            } => {
                let callee = self.eval(callee)?;
                let mut args = Vec::new();
                for arg in arguments {
                    args.push(self.eval(arg)?);
                }

                match callee {
                    Value::Native(f) => {
                        if f.arity() != args.len() {
                            return RuntimeError::Arity {
                                expected: f.arity(),
                                got: args.len(),
                            }
                            .into();
                        }

                        Ok(f.call(self, args)?)
                    }
                    Value::Function(f) => {
                        if f.arity() != args.len() {
                            return RuntimeError::Arity {
                                expected: f.arity(),
                                got: args.len(),
                            }
                            .into();
                        }
                        Ok(f.call(self, args)?)
                    }
                    _ => return RuntimeError::NotCallable(paren.lexeme.clone()).into(),
                }
            }

            _ => unimplemented!(),
        }
    }

    pub(crate) fn execute(&mut self, stmt: &Stmt) -> EvalResult<()> {
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

                self.environment
                    .borrow_mut()
                    .define(name.lexeme.clone(), value);
                Ok(())
            }
            Stmt::Block { statements } => {
                let environment = Environment::new()
                    .with_enclosing(self.environment.clone())
                    .rc();
                self.execute_block(statements, environment)
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if self.eval(condition)?.is_truthy() {
                    self.execute(then_branch)
                } else if let Some(else_b) = else_branch {
                    self.execute(else_b)
                } else {
                    Ok(())
                }
            }
            Stmt::While { condition, body } => {
                while self.eval(condition)?.is_truthy() {
                    self.execute(body)?;
                }
                Ok(())
            }
            Stmt::Function { name, params, body } => {
                let function = Value::Function(Function {
                    declaration: Stmt::Function {
                        name: name.clone(),
                        params: params.clone(),
                        body: body.clone(),
                    },
                    closure: self.environment.clone(),
                });
                self.environment
                    .borrow_mut()
                    .define(name.lexeme.to_string(), Some(function));
                Ok(())
            }
            Stmt::Return { keyword: _, value } => {
                let value = if let Some(v) = value {
                    self.eval(v)?
                } else {
                    Value::Nil
                };

                Err(RuntimeControl::Return(value))
            }

            _ => unimplemented!(),
        }
    }

    pub(crate) fn execute_block(
        &mut self,
        statements: &[Stmt],
        environment: Rc<RefCell<Environment>>,
    ) -> EvalResult<()> {
        let previous = self.environment.clone();
        self.environment = environment;

        let result: EvalResult<()> = (|| {
            for stmt in statements {
                self.execute(stmt)?;
            }
            Ok(())
        })();

        self.environment = previous;
        result
    }

    fn look_up_var(&mut self, name: &Token, expr: &Expr) -> RuntimeResult<Value> {
        let expr_id = expr.id().expect("variable expressions always have an id");
        if let Some(distance) = self.locals.get(&expr_id) {
            self.environment.borrow().get_at(*distance, &name.lexeme)
        } else {
            self.globals.borrow().get(name)
        }
    }
}

pub(crate) trait Callable: ToString {
    fn call(self, interpreter: &mut Interpreter, args: Vec<Value>) -> RuntimeResult<Value>;
    fn arity(&self) -> usize;
}
