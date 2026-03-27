use crate::{Environment, Expr, IsTruthy, RuntimeResult, Stmt, TokenType, Value};
use std::{cell::RefCell, ops::Deref, rc::Rc};

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Rc::new(RefCell::new(Environment::new())),
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
            Variable { name } => self.environment.borrow().get(name),
            Assign { name, value } => {
                let value = self.eval(&value)?;
                self.environment.borrow_mut().assign(name, &value)?;
                Ok(value)
            }
            Logical {
                left,
                operator,
                right,
            } => {
                let left = self.eval(&left)?;
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

                self.environment
                    .borrow_mut()
                    .define(name.lexeme.clone(), value);
                Ok(())
            }
            Stmt::Block { statements } => {
                let prev = self.environment.clone();
                self.environment = Environment::new().with_enclosing(prev.clone()).rc();

                let res: RuntimeResult<()> = (|| {
                    for stmt in statements {
                        self.execute(stmt)?;
                    }
                    Ok(())
                })();

                self.environment = prev;
                res
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if self.eval(&condition)?.is_truthy() {
                    self.execute(&then_branch)
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

            _ => unimplemented!(),
        }
    }
}
