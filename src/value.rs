use crate::{Callable, Environment, Literal, RuntimeControl, RuntimeError, RuntimeResult, Stmt};
use std::{
    cmp::Ordering,
    fmt::Display,
    ops::{Add, Div, Mul, Neg, Not, Sub},
    time::UNIX_EPOCH,
};

#[derive(PartialEq, Debug, Clone, Default)]
pub enum Value {
    #[default]
    Nil,
    Bool(bool),
    Number(f64),
    String(String),
    Native(NativeFunction),
    Function(Function),
}

// TODO: TRANSFORM INTO AN ENUM
#[derive(PartialEq, Debug, Clone, Default)]
pub struct NativeFunction {
    arity: usize,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Function {
    pub declaration: Stmt,
}

impl Callable for Function {
    fn arity(&self) -> usize {
        match &self.declaration {
            Stmt::Function {
                name: _,
                params,
                body: _,
            } => params.len(),
            _ => unreachable!(),
        }
    }
    fn call(self, interpreter: &mut crate::Interpreter, args: Vec<Value>) -> RuntimeResult<Value> {
        match self.declaration {
            Stmt::Function {
                name: _,
                params,
                body,
            } => {
                let prev = interpreter.environment.clone();
                interpreter.environment = Environment::new().with_enclosing(prev.clone()).rc();

                for (i, p) in params.iter().enumerate() {
                    interpreter
                        .environment
                        .borrow_mut()
                        .define(p.lexeme.clone(), args.get(i).cloned());
                }

                let result = interpreter.execute(&Stmt::Block { statements: body });
                interpreter.environment = prev;

                match result {
                    Ok(()) => Ok(Value::Nil),
                    Err(RuntimeControl::Return(value)) => Ok(value),
                    Err(RuntimeControl::Error(err)) => Err(err),
                }
            }
            _ => {
                return Err(RuntimeError::NotCallable(
                    "Tried to call function with no declaration".into(),
                ));
            }
        }
    }
}

impl ToString for NativeFunction {
    fn to_string(&self) -> String {
        String::from("<native fn>")
    }
}

impl ToString for Function {
    fn to_string(&self) -> String {
        match &self.declaration {
            Stmt::Function {
                name,
                params: _,
                body: _,
            } => format!("<fn {}>", name.lexeme),
            _ => unreachable!(),
        }
    }
}
impl Callable for NativeFunction {
    fn call(
        self,
        _interpreter: &mut crate::Interpreter,
        _args: Vec<Value>,
    ) -> RuntimeResult<Value> {
        Ok(Value::Number(
            std::time::SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Safe to expect.")
                .as_secs_f64(),
        ))
    }
    fn arity(&self) -> usize {
        self.arity
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Value::*;
        match self {
            Number(n) => {
                let text = n.to_string();
                let trimmed = text.trim_end_matches(".0");
                write!(f, "{trimmed}")
            }
            String(s) => {
                write!(f, "{s}")
            }
            Bool(b) => write!(f, "{b}"),
            Nil => write!(f, "nil"),
            Native(n) => write!(f, "{}", n.to_string()),
            Function(d) => write!(f, "{}", d.to_string()),
        }
    }
}
impl From<&Literal> for Value {
    fn from(value: &Literal) -> Self {
        use Literal::*;
        match value {
            Nil => Self::Nil,
            Bool(b) => Self::Bool(*b),
            Number(n) => Self::Number(*n),
            String(s) => Self::String(s.clone()),
        }
    }
}

impl Not for Value {
    type Output = Self;
    fn not(self) -> Self::Output {
        Self::Bool(!self.is_truthy())
    }
}

impl Neg for Value {
    type Output = RuntimeResult<Self>;

    fn neg(self) -> Self::Output {
        match self {
            Self::Number(a) => Ok(Value::Number(-a)),
            _ => Err(RuntimeError::TypeError {
                message: "Tried to apply '-' operator on a non-number".to_string(),
                value: self,
            }),
        }
    }
}

impl From<&Literal> for RuntimeResult<Value> {
    fn from(value: &Literal) -> Self {
        Ok(value.into())
    }
}

pub trait IsTruthy {
    fn is_truthy(&self) -> bool;
}

impl IsTruthy for Value {
    fn is_truthy(&self) -> bool {
        use Value::*;
        match self {
            Nil => false,
            Bool(b) => *b,
            _ => true,
        }
    }
}

impl Add for Value {
    type Output = RuntimeResult<Self>;
    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(a), Self::Number(b)) => Ok(Self::Number(a + b)),
            (Self::String(a), Self::String(b)) => Ok(Self::String(a + &b)),
            (left, _right) => Err(RuntimeError::TypeError {
                message: "Operands must be either strings or numbers".to_string(),
                value: left,
            }),
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::String(value.to_string())
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Number(value)
    }
}

impl<T> From<Option<T>> for Value
where
    T: Into<Value>,
{
    fn from(value: Option<T>) -> Self {
        match value {
            Some(v) => v.into(),
            None => Value::Nil,
        }
    }
}

impl From<()> for Value {
    fn from(_: ()) -> Self {
        Value::Nil
    }
}

macro_rules! impl_numeric_binop {
    ($trait:ident, $method:ident, $op:tt) => {
        impl $trait for Value {
            type Output = RuntimeResult<Self>;

            fn $method(self, rhs: Self) -> Self::Output {
                match (self, rhs) {
                    (Self::Number(a), Self::Number(b)) => Ok(Self::Number(a $op b)),
                    (left, _right) => Err(RuntimeError::TypeError {
                    message:    "Operands must be numbers".to_string(),
                    value: left
                    }),
                }
            }
        }
    };
}

impl_numeric_binop!(Sub, sub, -);
impl_numeric_binop!(Div, div, /);
impl_numeric_binop!(Mul, mul, *);
