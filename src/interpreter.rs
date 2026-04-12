use crate::{
    Class, Environment, EvalResult, Expr, Function, Instance, IsTruthy, RuntimeControl,
    RuntimeError, RuntimeResult, Stmt, Token, TokenType, Value,
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
            .expect("resolver only stores locals for expressions with ids");
        self.locals.insert(id, depth);
    }

    fn eval(&mut self, expr: &Expr) -> EvalResult<Value> {
        use Expr::*;

        match expr {
            Literal { value } => Ok(value.into()),
            Grouping { expression } => self.eval(expression),
            Unary { operator, right } => {
                let right = self.eval(right)?;
                match operator.token_type {
                    TokenType::Minus => Ok((-right)?),
                    TokenType::Bang => Ok(!right),
                    _ => unreachable!(),
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
                    TokenType::Minus => Ok((left - right)?),
                    TokenType::Slash => Ok((left / right)?),
                    TokenType::Star => Ok((left * right)?),
                    TokenType::Plus => Ok((left + right)?),
                    TokenType::Greater => Ok((left > right).into()),
                    TokenType::GreaterEqual => Ok((left >= right).into()),
                    TokenType::Less => Ok((left < right).into()),
                    TokenType::LessEqual => Ok((left <= right).into()),
                    TokenType::BangEqual => Ok((left != right).into()),
                    TokenType::EqualEqual => Ok((left == right).into()),
                    _ => unreachable!(),
                }
            }
            Variable { name, .. } | This { keyword: name, .. } => Ok(self.look_up_var(name, expr)?),
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
                } else if !left.is_truthy() {
                    Ok(left)
                } else {
                    self.eval(right)
                }
            }
            Call {
                callee,
                paren,
                arguments,
            } => {
                let callee = self.eval(callee)?;
                let mut args = Vec::with_capacity(arguments.len());
                for argument in arguments {
                    args.push(self.eval(argument)?);
                }

                match callee {
                    Value::Native(function) => {
                        if function.arity() != args.len() {
                            return RuntimeError::Arity {
                                expected: function.arity(),
                                got: args.len(),
                            }
                            .into();
                        }

                        Ok(function.call(self, args)?)
                    }
                    Value::Function(function) => {
                        if function.arity() != args.len() {
                            return RuntimeError::Arity {
                                expected: function.arity(),
                                got: args.len(),
                            }
                            .into();
                        }

                        Ok(function.call(self, args)?)
                    }
                    Value::Class(class) => {
                        if class.arity() != args.len() {
                            return RuntimeError::Arity {
                                expected: class.arity(),
                                got: args.len(),
                            }
                            .into();
                        }

                        Ok(class.call(self, args)?)
                    }
                    _ => RuntimeError::NotCallable(paren.lexeme.clone()).into(),
                }
            }
            Get { object, name } => {
                let object = self.eval(object)?;
                match object {
                    Value::Instance(instance) => Ok(Instance::get(&instance, name)?),
                    _ => RuntimeError::PropertiesOnInstancesOnly.into(),
                }
            }
            Set {
                object,
                name,
                value,
            } => {
                let object = self.eval(object)?;
                let value = self.eval(value)?;

                match object {
                    Value::Instance(instance) => {
                        instance.borrow_mut().set(name, value.clone());
                        Ok(value)
                    }
                    _ => RuntimeError::FieldsOnInstancesOnly.into(),
                }
            }
            Super { method, .. } => {
                let distance = *self
                    .locals
                    .get(&expr.id().expect("super expressions always have an id"))
                    .expect("resolver must define a scope distance for super");

                let superclass = match self.environment.borrow().get_at(distance, "super")? {
                    Value::Class(class) => class,
                    _ => unreachable!(),
                };

                let object = match self.environment.borrow().get_at(distance - 1, "this")? {
                    Value::Instance(instance) => instance,
                    _ => unreachable!(),
                };

                match superclass.find_method(&method.lexeme) {
                    Some(method) => Ok(Value::Function(method.bind(&object))),
                    None => RuntimeError::Undefined {
                        lexeme: method.lexeme.clone(),
                    }
                    .into(),
                }
            }
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
                let value = if let Some(initializer) = initializer.deref() {
                    Some(self.eval(initializer)?)
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
                } else if let Some(else_branch) = else_branch {
                    self.execute(else_branch)
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
            Stmt::Function { name, .. } => {
                let function = Value::Function(Function {
                    declaration: stmt.clone(),
                    closure: self.environment.clone(),
                    is_initializer: false,
                });

                self.environment
                    .borrow_mut()
                    .define(name.lexeme.clone(), Some(function));
                Ok(())
            }
            Stmt::Return { value, .. } => {
                let value = if let Some(value) = value {
                    self.eval(value)?
                } else {
                    Value::Nil
                };

                Err(RuntimeControl::Return(value))
            }
            Stmt::Class {
                name,
                super_class,
                methods,
            } => {
                let superclass = if let Some(super_class) = super_class {
                    match self.eval(super_class)? {
                        Value::Class(class) => Some(Rc::new(class)),
                        _ => return RuntimeError::SuperclassMustBeClass.into(),
                    }
                } else {
                    None
                };

                self.environment
                    .borrow_mut()
                    .define(name.lexeme.clone(), None);

                let enclosing_environment = self.environment.clone();

                if let Some(superclass) = &superclass {
                    let environment = Environment::new()
                        .with_enclosing(self.environment.clone())
                        .rc();
                    environment.borrow_mut().define(
                        "super".to_string(),
                        Some(Value::Class((**superclass).clone())),
                    );
                    self.environment = environment;
                }

                let mut class_methods = HashMap::new();
                for method in methods {
                    match method {
                        Stmt::Function { name, .. } => {
                            class_methods.insert(
                                name.lexeme.clone(),
                                Function {
                                    declaration: method.clone(),
                                    closure: self.environment.clone(),
                                    is_initializer: name.lexeme == "init",
                                },
                            );
                        }
                        _ => unreachable!(),
                    }
                }

                if superclass.is_some() {
                    self.environment = enclosing_environment.clone();
                }

                let class = Value::Class(Class::new(
                    name.lexeme.clone(),
                    superclass,
                    class_methods,
                ));

                self.environment.borrow_mut().assign(name, &class)?;
                Ok(())
            }
        }
    }

    pub(crate) fn execute_block(
        &mut self,
        statements: &[Stmt],
        environment: Rc<RefCell<Environment>>,
    ) -> EvalResult<()> {
        let previous = self.environment.clone();
        self.environment = environment;

        let result = (|| {
            for stmt in statements {
                self.execute(stmt)?;
            }
            Ok(())
        })();

        self.environment = previous;
        result
    }

    fn look_up_var(&mut self, name: &Token, expr: &Expr) -> RuntimeResult<Value> {
        let expr_id = expr.id().expect("resolved expressions always have an id");
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
