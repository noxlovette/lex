use crate::{Literal, RuntimeError, RuntimeResult};
use std::ops::{Div, Mul, Neg, Not, Sub};

pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    String(String),
}

impl From<Literal> for Value {
    fn from(value: Literal) -> Self {
        use Literal::*;
        match value {
            Nil => Self::Nil,
            Bool(b) => Self::Bool(b),
            Number(n) => Self::Number(n),
            String(s) => Self::String(s),
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
            _ => Err(RuntimeError::TypeError(
                "Tried to apply '-' operator on a non-number".to_string(),
            )),
        }
    }
}

impl From<Literal> for RuntimeResult<Value> {
    fn from(value: Literal) -> Self {
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
macro_rules! impl_numeric_binop {
    ($trait:ident, $method:ident, $op:tt) => {
        impl $trait for Value {
            type Output = RuntimeResult<Self>;

            fn $method(self, rhs: Self) -> Self::Output {
                match (self, rhs) {
                    (Self::Number(a), Self::Number(b)) => Ok(Self::Number(a $op b)),
                    _ => Err(RuntimeError::TypeError(
                        "Operands must be numbers".to_string(),
                    )),
                }
            }
        }
    };
}

impl_numeric_binop!(Sub, sub, -);
impl_numeric_binop!(Div, div, /);
impl_numeric_binop!(Mul, mul, *);
